use bot_core::Error;
use tokio::signal::unix::{SignalKind, signal as unix_signal};

pub async fn signal() -> Result<(), Error> {
    let mut terminate = unix_signal(SignalKind::terminate())?;
    tokio::select! {
        result = tokio::signal::ctrl_c() => result?,
        _ = terminate.recv() => {},
    }
    Ok(())
}
