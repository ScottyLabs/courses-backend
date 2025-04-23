use tokio::signal;

#[cfg(feature = "tls")]
use axum_server::Handle;

/// Listens for shutdown signals (Ctrl+C or Unix signals)
#[cfg(feature = "tls")]
pub async fn shutdown_signal(handle: Handle) {
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

    // Trigger graceful shutdown
    handle.graceful_shutdown(None);
}

/// Listens for shutdown signals (Ctrl+C or Unix signals)
#[cfg(not(feature = "tls"))]
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
