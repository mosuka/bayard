use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use log::*;
use raft::eraftpb::{ConfChange, Entry, EntryType, Message};
use raft::storage::MemStorage as PeerStorage;
use raft::{self, RawNode};

use crate::server::util;

pub enum PeerMessage {
    Propose(Vec<u8>),
    Message(Message),
    ConfChange(ConfChange),
}

pub struct Peer {
    raft_group: RawNode<PeerStorage>,
    last_applying_idx: u64,
    last_compacted_idx: u64,
    apply_ch: SyncSender<Entry>,
    peers_addr: HashMap<u64, (String, u32)>,
}

impl Peer {
    pub fn new(id: u64, apply_ch: SyncSender<Entry>, peers: Vec<u64>) -> Peer {
        let cfg = util::default_raft_config(id, peers);
        let storge = PeerStorage::new();
        Peer {
            raft_group: RawNode::new(&cfg, storge, vec![]).unwrap(),
            last_applying_idx: 0,
            last_compacted_idx: 0,
            apply_ch,
            peers_addr: HashMap::new(),
        }
    }

    pub fn activate(mut peer: Peer, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        thread::spawn(move || {
            peer.listen_message(sender, receiver);
        });
    }

    fn listen_message(&mut self, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        let mut t = Instant::now();
        let mut timeout = Duration::from_millis(100);
        loop {
            match receiver.recv_timeout(timeout) {
                Ok(PeerMessage::Propose(p)) => match self.raft_group.propose(vec![], p) {
                    Ok(_) => (),
                    Err(_) => self.apply_message(Entry::new()),
                },
                Ok(PeerMessage::ConfChange(cc)) => {
                    match self.raft_group.propose_conf_change(vec![], cc.clone()) {
                        Ok(_) => (),
                        Err(_) => debug!("conf change failed: {:?}", cc),
                    }
                }
                Ok(PeerMessage::Message(m)) => self.raft_group.step(m).unwrap(),
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => return,
            }

            let d = t.elapsed();
            if d >= timeout {
                t = Instant::now();
                timeout = Duration::from_millis(200);
                self.raft_group.tick();
            } else {
                timeout -= d;
            }

            self.on_ready(sender.clone());
        }
    }

    fn on_ready(&mut self, sender: SyncSender<Message>) {
        if !self.raft_group.has_ready() {
            return;
        }

        let mut ready = self.raft_group.ready();
        let is_leader = self.raft_group.raft.leader_id == self.raft_group.raft.id;
        if is_leader {
            //            debug!("I'm leader");
            let msgs = ready.messages.drain(..);
            for _msg in msgs {
                Self::send_message(sender.clone(), _msg.clone());
            }
        }

        if !raft::is_empty_snap(&ready.snapshot) {
            self.raft_group
                .mut_store()
                .wl()
                .apply_snapshot(ready.snapshot.clone())
                .unwrap()
        }

        if !ready.entries.is_empty() {
            self.raft_group
                .mut_store()
                .wl()
                .append(&ready.entries)
                .unwrap();
        }

        if let Some(ref hs) = ready.hs {
            self.raft_group.mut_store().wl().set_hardstate(hs.clone());
        }

        if !is_leader {
            //            debug!("I'm follower");
            let msgs = ready.messages.drain(..);
            for mut _msg in msgs {
                for _entry in _msg.mut_entries().iter() {
                    if _entry.get_entry_type() == EntryType::EntryConfChange {}
                }
                Self::send_message(sender.clone(), _msg.clone());
            }
        }

        if let Some(committed_entries) = ready.committed_entries.take() {
            let mut _last_apply_index = 0;
            for entry in committed_entries {
                // Mostly, you need to save the last apply index to resume applying
                // after restart. Here we just ignore this because we use a Memory storage.
                _last_apply_index = entry.get_index();

                if entry.get_data().is_empty() {
                    // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                    continue;
                }

                match entry.get_entry_type() {
                    EntryType::EntryNormal => self.apply_message(entry.clone()),
                    EntryType::EntryConfChange => {
                        let cc = util::parse_data(&entry.data);
                        self.raft_group.apply_conf_change(&cc);
                        debug!("apply conf change");
                        self.apply_message(entry.clone());
                    }
                }
            }
        }

        // Advance the Raft
        self.raft_group.advance(ready);
    }

    fn send_message(sender: SyncSender<Message>, msg: Message) {
        thread::spawn(move || {
            sender.send(msg).unwrap_or_else(|e| {
                panic!("raft send message error: {}", e);
            });
        });
    }

    fn apply_message(&self, entry: Entry) {
        let sender = self.apply_ch.clone();
        thread::spawn(move || {
            sender.send(entry).unwrap_or_else(|e| {
                panic!("raft send apply entry error: {}", e);
            });
        });
    }
}
