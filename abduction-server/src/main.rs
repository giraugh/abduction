use futures::{stream, Stream, StreamExt};
use qubit::{handler, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// The context type for qubit
#[derive(Debug, Clone)]
struct QubitCtx;

#[handler(subscription)]
async fn events_stream(_ctx: QubitCtx) -> impl Stream<Item = String> {
    stream::iter(0..).then(async move |x| {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        x.to_string()
    })
}

#[tokio::main]
async fn main() {
    // Init tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // Create a qubit router
    let router = Router::new().handler(events_stream);

    // Generate ts types
    info!("Writing ts bindings");
    router
        .generate_type("../abduction-site/src/lib/api.gen.ts")
        .expect("Failed to write bindings");

    // Create service and handle
    let (qubit_service, qubit_handle) = router.into_service(QubitCtx);

    // Nest into an Axum router
    let axum_router = axum::Router::<()>::new().nest_service("/rpc", qubit_service);

    // Start a Hyper server
    info!("Listening at 127.0.0.1:9944");
    axum::serve(
        TcpListener::bind(&SocketAddr::from(([127, 0, 0, 1], 9944)))
            .await
            .unwrap(),
        axum_router,
    )
    .await
    .unwrap();

    // Shutdown Qubit
    qubit_handle.stop().unwrap();
}
