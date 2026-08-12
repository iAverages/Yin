use bot_core::Error;

pub async fn signal() -> Result<(), Error> {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await?;
        Ok::<(), std::io::Error>(())
    };

    #[cfg(unix)]
    {
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?
                .recv()
                .await;
            Ok::<(), std::io::Error>(())
        };

        tokio::select! {
            result = ctrl_c => result?,
            result = terminate => result?,
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await?;
    }

    Ok(())
}
