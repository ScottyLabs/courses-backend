use tokio::signal;

/// Listens for shutdown signals (Ctrl+C or Unix signals)
pub async fn shutdown_signal() {
    let ctrl_c = async { signal::ctrl_c().await };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
