use std::{
    fmt,
    net::SocketAddr,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

use bytes::{BufMut, Bytes, BytesMut};
use foca::{Config, Foca, Notification, PostcardCodec};
use rand::{prelude::StdRng, SeedableRng};
use tokio::{
    net::UdpSocket,
    sync::{mpsc, watch, RwLock, RwLockReadGuard},
};
use tokio_stream::wrappers::WatchStream;
use tracing::{debug, error, info};

use crate::common::write_file;

use super::{
    broadcast::MessageHandler,
    member::Member,
    members::Members,
    message::{Input, Message},
    metadata::Metadata,
    runtime::AccumulatingRuntime,
};

const FOCA_CHANNEL_BUFFER_SIZE: usize = 100;
const MEMBERS_FILE: &str = "members.json";

#[derive(Debug, Clone, Copy)]
pub enum MembershipErrorKind {
    SocketBindingFailure,
    MembersSerializationFailure,
    BroadcastFailure,
    FileWriteFailure,
}

impl MembershipErrorKind {
    pub fn with_error<E>(self, source: E) -> MembershipError
    where
        anyhow::Error: From<E>,
    {
        MembershipError {
            kind: self,
            source: From::from(source),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("MembershipError(kind={kind:?}, source={source})")]
pub struct MembershipError {
    pub kind: MembershipErrorKind,
    #[source]
    source: anyhow::Error,
}

impl MembershipError {
    pub fn add_context<C>(self, ctx: C) -> Self
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        MembershipError {
            kind: self.kind,
            source: self.source.context(ctx),
        }
    }

    pub fn kind(&self) -> MembershipErrorKind {
        self.kind
    }
}

async fn save_members(path: &Path, members: Members) -> Result<(), MembershipError> {
    let members_bytes = serde_json::to_vec(&members)
        .map_err(|error| MembershipErrorKind::MembersSerializationFailure.with_error(error))?;

    write_file(path, members_bytes.as_slice())
        .await
        .map_err(|error| MembershipErrorKind::FileWriteFailure.with_error(error))
}

pub struct Membership {
    foca: Arc<RwLock<Foca<Member, PostcardCodec, StdRng, MessageHandler>>>,
    local_member: Member,
    members: Arc<RwLock<Members>>,
    members_receiver: watch::Receiver<Members>,
    message_receiver: watch::Receiver<Message>,
}

impl Membership {
    pub async fn new(
        bind_address: SocketAddr,
        advertise_address: SocketAddr,
        metadata: Metadata,
        data_directory: PathBuf,
        seed_address: Option<SocketAddr>,
    ) -> Result<Self, MembershipError> {
        // Create a config specifies the parameters Foca will use for the SWIM protocol.
        let mut config = Config::simple();
        config.max_packet_size = NonZeroUsize::new(1024 * 1024).unwrap();

        // Create a node identity.
        let local_member = Member::new_with_metadata(advertise_address, metadata);

        // Create members channel.
        let (members_sender, members_receiver) =
            watch::channel::<Members>(Members::init(vec![local_member.clone()]));

        // Create message channel.
        let (message_sender, message_receiver) = watch::channel::<Message>(Message::default());

        // Create a Foca instance.
        let foca = Arc::new(RwLock::new(Foca::with_custom_broadcast(
            local_member.clone(),
            config.clone(),
            StdRng::from_entropy(),
            PostcardCodec,
            MessageHandler::new(message_sender),
        )));

        // Create remote member list with initial identity list.
        let members = Arc::new(RwLock::new(Members::init(vec![local_member.clone()])));

        // Binding UDP socket.
        let socket = Arc::new(
            UdpSocket::bind(bind_address)
                .await
                .map_err(|error| MembershipErrorKind::SocketBindingFailure.with_error(error))?,
        );

        // We'll create a task responsible to sending data through the socket.
        // These are what we use to communicate with it.
        let (tx_data, mut rx_data) = mpsc::channel::<(SocketAddr, Bytes)>(100);

        // The socket writing task
        let write_socket = Arc::clone(&socket);
        tokio::spawn(async move {
            info!(?write_socket, "Starting socket writing task.");
            while let Some((dst, data)) = rx_data.recv().await {
                // A more reasonable implementation would do some more stuff
                // here before sending, like:
                //  * zlib or something else to compress the data
                //  * encryption (shared key, AES most likely)
                //  * an envelope with tag+version+checksum to allow
                //    protocol evolution
                let _ignored_send_result = write_socket.send_to(&data, &dst).await;
            }
        });

        // And communicating via channels
        let (tx_foca, mut rx_foca): (mpsc::Sender<Input<Member>>, mpsc::Receiver<Input<Member>>) =
            mpsc::channel(FOCA_CHANNEL_BUFFER_SIZE);

        // Another alternative would be putting a Lock around Foca,
        // but yours truly likes to hide behind (the lock inside) channels instead.
        let mut runtime: AccumulatingRuntime<Member> = AccumulatingRuntime::new();
        let tx_foca_task = tx_foca.clone();
        let members_task = Arc::clone(&members);
        let foca_task = Arc::clone(&foca);
        let data_directory_task = data_directory.clone();

        tokio::spawn(async move {
            while let Some(input) = rx_foca.recv().await {
                debug_assert_eq!(0, runtime.backlog());

                let result = match input {
                    Input::Event(timer) => {
                        debug!(?timer, "Received timer.");
                        foca_task.write().await.handle_timer(timer, &mut runtime)
                    }
                    Input::Data(data) => {
                        debug!(?data, "Received data.");
                        foca_task.write().await.handle_data(&data, &mut runtime)
                    }
                    Input::Announce(dst) => {
                        debug!(?dst, "Received announce.");
                        foca_task.write().await.announce(dst, &mut runtime)
                    }
                };

                // Every public foca result yields `()` on success, so there's
                // nothing to do with Ok
                if let Err(error) = result {
                    // And we'd decide what to do with each error,
                    // but Foca is pretty tolerant so we just log them and pretend all is fine
                    error!(?error, "Ignored Error.");
                }

                // Now we react to what happened.
                // This is how we enable async: buffer one single interaction
                // and then drain the runtime.

                // First we submit everything that needs to go to the network
                while let Some((dst, data)) = runtime.to_send.pop() {
                    // ToSocketAddrs would be the fancy thing to use here
                    let _ignored_send_result = tx_data.send((dst.addr, data)).await;
                }

                // Then schedule what needs to be scheduled
                while let Some((delay, event)) = runtime.to_schedule.pop() {
                    let own_input_handle = tx_foca_task.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(delay).await;
                        let _ignored_send_error = own_input_handle.send(Input::Event(event)).await;
                    });
                }

                // And finally react to notifications.
                //
                // Here we could do smarter things to keep other actors in
                // the system up-to-date with the cluster state.
                // We could, for example:
                //
                //  * Have a broadcast channel where we submit the MemberUp
                //    and MemberDown notifications to everyone and each one
                //    keeps a lock-free version of the list
                //
                //  * Update a shared/locked Vec that every consumer has
                //    read access
                //
                // But since this is an agent, we simply write to a file
                // so other proccesses periodically open()/read()/close()
                // to figure out the cluster members.
                let mut active_list_has_changed = false;
                while let Some(notification) = runtime.notifications.pop() {
                    match notification {
                        Notification::MemberUp(member) => {
                            info!(?member, "Member up.");
                            active_list_has_changed |=
                                members_task.write().await.push(member).is_some()
                        }
                        Notification::MemberDown(member) => {
                            info!(?member, "Member down.");
                            active_list_has_changed |=
                                members_task.write().await.remove(&member.addr).is_some()
                        }
                        other => {
                            info!(notification = ?other, "Receive membership notification.");
                        }
                    }
                }

                if active_list_has_changed {
                    let members_copy = members_task.read().await.clone();
                    info!(members = ?members_copy, "Active members has changed.");

                    match save_members(
                        &data_directory_task.join(MEMBERS_FILE),
                        members_copy.clone(),
                    )
                    .await
                    {
                        Ok(_) => info!("Saved active members."),
                        Err(error) => {
                            error!(?error, "Failed to save active members.");
                            continue;
                        }
                    }

                    match members_sender.send(members_copy) {
                        Ok(_) => info!("Sent active members."),
                        Err(error) => {
                            error!(?error, "Failed to send active members.");
                            continue;
                        }
                    }
                }
            }
        });

        // Foca is running, we can tell it to announce to our target
        if let Some(seed_address) = seed_address {
            let seed_member = Member::new(seed_address);
            match tx_foca.send(Input::Announce(seed_member.clone())).await {
                Ok(_) => info!(?seed_member, "Announced to seed."),
                Err(error) => {
                    error!("Failed to announce to seed. {}", error);
                }
            }
        }

        // And finally, we receive forever
        let mut recv_buf = vec![0u8; config.max_packet_size.get()];
        tokio::spawn(async move {
            info!(?socket, "Listening on.");
            let mut databuf = BytesMut::new();
            loop {
                let (len, _from_addr) = match socket.recv_from(&mut recv_buf).await {
                    Ok(result) => result,
                    Err(error) => {
                        error!(?error, "Error receiving.");
                        continue;
                    }
                };

                // Accordinly, we would undo everything that's done prior to
                // sending: decompress, decrypt, remove the envelope
                databuf.put_slice(&recv_buf[..len]);

                // And simply forward it to foca
                let _ignored_send_error = tx_foca.send(Input::Data(databuf.split().freeze())).await;
            }
        });

        let membership = Self {
            foca,
            local_member,
            members,
            members_receiver,
            message_receiver,
        };

        Ok(membership)
    }

    pub fn watch_members(&self) -> WatchStream<Members> {
        WatchStream::new(self.members_receiver.clone())
    }

    pub fn watch_message(&self) -> WatchStream<Message> {
        WatchStream::new(self.message_receiver.clone())
    }

    pub async fn local_member(&self) -> Member {
        self.local_member.clone()
    }

    pub async fn remote_members(&self) -> Vec<Member> {
        self.members
            .read()
            .await
            .iter()
            .cloned()
            .filter(|id| id != &self.local_member)
            .collect::<Vec<Member>>()
    }

    pub async fn members(&self) -> RwLockReadGuard<Members> {
        self.members.read().await
    }

    pub async fn broadcast(&self, message: Message) -> Result<(), MembershipError> {
        self.foca
            .write()
            .await
            .add_broadcast(message.as_ref())
            .map_err(|error| {
                MembershipErrorKind::BroadcastFailure.with_error(anyhow::anyhow!(error))
            })
    }
}
