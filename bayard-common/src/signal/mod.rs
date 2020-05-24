use crossbeam_channel::{bounded, Receiver};
use ctrlc;
use tokio;

pub fn sigterm_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })
    .unwrap();

    Ok(receiver)
}

pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
