// use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use log::*;
use raft::{self, RawNode};
use raft::eraftpb::{ConfChange, ConfState, Entry, EntryType, Message, MessageType};
use raft::storage::MemStorage;

use crate::server::util;

pub enum PeerMessage {
    Propose(Vec<u8>),
    Message(Message),
    ConfChange(ConfChange),
}

pub struct Peer {
    pub raw_node: RawNode<MemStorage>,
    last_apply_index: u64,
    // last_compacted_idx: u64,
    conf_state: Option<ConfState>,
    apply_ch: SyncSender<Entry>,
    // peers_addr: HashMap<u64, (String, u32)>, // id, (host, port)
}

impl Peer {
    pub fn new(id: u64, apply_ch: SyncSender<Entry>, peers: Vec<u64>) -> Peer {
        let cfg = util::default_raft_config(id, peers);
        let storge = MemStorage::new();
        let peer = Peer {
            raw_node: RawNode::new(&cfg, storge, vec![]).unwrap(),
            last_apply_index: 0,
            // last_compacted_idx: 0,
            conf_state: None,
            apply_ch,
            // peers_addr: HashMap::new(),
        };
        peer
    }

    pub fn activate(mut peer: Peer, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        thread::spawn(move || {
            peer.listen_message(sender, receiver);
        });
    }

    fn listen_message(&mut self, sender: SyncSender<Message>, receiver: Receiver<PeerMessage>) {
        debug!("start listening message");

        let mut t = Instant::now();
        let mut timeout = Duration::from_millis(100);
        loop {
            match receiver.recv_timeout(timeout) {
                Ok(PeerMessage::Propose(p)) => match self.raw_node.propose(vec![], p.clone()) {
                    Ok(_) => info!("proposal succeeded: {:?}", p),
                    Err(_) => {
                        warn!("proposal failed: {:?}", p);
                        self.apply_message(Entry::new());
                    }
                },
                Ok(PeerMessage::ConfChange(cc)) => {
                    match self.raw_node.propose_conf_change(vec![], cc.clone()) {
                        Ok(_) => info!("proposed configuration change succeeded: {:?}", cc),
                        Err(_) => warn!("proposed configuration change failed: {:?}", cc),
                    }
                }
                Ok(PeerMessage::Message(m)) => self.raw_node.step(m).unwrap(),
                Err(RecvTimeoutError::Timeout) => (),
                Err(RecvTimeoutError::Disconnected) => return,
            }

            let d = t.elapsed();
            if d >= timeout {
                t = Instant::now();
                timeout = Duration::from_millis(200);
                self.raw_node.tick();
            } else {
                timeout -= d;
            }

            self.on_ready(sender.clone());
        }
    }

    fn is_leader(&self) -> bool {
        self.raw_node.raft.leader_id == self.raw_node.raft.id
    }

    fn on_ready(&mut self, sender: SyncSender<Message>) {
        if !self.raw_node.has_ready() {
            return;
        }
        let mut ready = self.raw_node.ready();

        let is_leader = self.is_leader();

        // leader
        if is_leader {
            let msgs = ready.messages.drain(..);
            for msg in msgs {
                Self::send_message(sender.clone(), msg.clone());
            }
        }

        if !raft::is_empty_snap(&ready.snapshot) {
            self.raw_node
                .mut_store()
                .wl()
                .apply_snapshot(ready.snapshot.clone())
                .unwrap()
        }

        if !ready.entries.is_empty() {
            self.raw_node
                .mut_store()
                .wl()
                .append(&ready.entries)
                .unwrap();
        }

        if let Some(ref hs) = ready.hs {
            self.raw_node.mut_store().wl().set_hardstate(hs.clone());
        }

        // not leader
        if !is_leader {
            let msgs = ready.messages.drain(..);
            for msg in msgs {
                Self::send_message(sender.clone(), msg.clone());
            }
        }

        if let Some(committed_entries) = ready.committed_entries.take() {
            for entry in committed_entries {
                // Mostly, you need to save the last apply index to resume applying
                // after restart. Here we just ignore this because we use a Memory storage.
                self.last_apply_index = entry.get_index();

                if entry.get_data().is_empty() {
                    // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                    continue;
                }

                match entry.get_entry_type() {
                    EntryType::EntryNormal => self.apply_message(entry.clone()),
                    EntryType::EntryConfChange => {
                        let cc = util::parse_data(&entry.data);
                        info!("apply config change: {:?}", cc);
                        self.raw_node.apply_conf_change(&cc);
                        self.apply_message(entry.clone());

                        self.conf_state = Some(self.raw_node.apply_conf_change(&cc));
                    }
                }
            }

            // if last_apply_index > 0 {
            //     info!("last apply index: {}", last_apply_index);
            //     let data = b"data".to_vec();
            //     match self.raw_node.mut_store().wl().create_snapshot(
            //         last_apply_index,
            //         conf_state,
            //         data,
            //     ) {
            //         Ok(_s) => (),
            //         Err(e) => error!("creating snapshot failed: {:?}", e),
            //     }
            // }
        }

        // Advance the Raft
        self.raw_node.advance(ready);

        // let raft_applied = self.raw_node.raft.raft_log.get_applied();
        // info!("raft_applied: {}", raft_applied);
        // match self.raw_node.mut_store().wl().compact(raft_applied) {
        //     Ok(_) => (),
        //     Err(e) => error!("compaction failed: {:?}", e),
        // }
    }

    fn send_message(sender: SyncSender<Message>, msg: Message) {
        thread::spawn(move || {
            // for entry in msg.mut_entries().iter() {
            //     debug!("leader: {:?}", entry);
            // }
            match msg.msg_type {
                MessageType::MsgHeartbeat => debug!("send message: {:?}", msg),
                MessageType::MsgHeartbeatResponse => debug!("send message: {:?}", msg),
                _ => info!("send message: {:?}", msg),
            }

            sender.send(msg).unwrap_or_else(|e| {
                panic!("error sending message: {:?}", e);
            });
        });
    }

    fn apply_message(&self, entry: Entry) {
        let sender = self.apply_ch.clone();
        thread::spawn(move || {
            info!("apply entry: {:?}", entry);

            sender.send(entry).unwrap_or_else(|e| {
                panic!("error sending apply entry: {:?}", e);
            });
        });
    }
}
