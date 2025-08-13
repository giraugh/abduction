mod entity;
mod mtch;
mod player_gen;

use futures::{Stream, StreamExt};
use qubit::{handler, Router};
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use std::{env, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::{
    entity::EntityManager,
    mtch::{MatchConfig, MatchManager, TickEvent},
};

const TICK_DELAY: Duration = Duration::from_secs(1);

pub type Db = Pool<Sqlite>;

/// The context type for qubit
#[derive(Debug, Clone)]
struct QubitCtx {
    tick_tx: broadcast::Sender<TickEvent>,
}

#[handler(subscription)]
async fn events_stream(ctx: QubitCtx) -> impl Stream<Item = TickEvent> {
    let stream = tokio_stream::wrappers::BroadcastStream::new(ctx.tick_tx.subscribe());
    stream.filter_map(|e| async { e.ok() })
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

    // Setup db connection
    let db_conn_string = env::var("DATABASE_URL")
        .expect("`DATABASE_URL` environment variable must contain a connection string");

    // DB
    let db = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(&db_conn_string)
            .unwrap()
            .create_if_missing(true),
    )
    .await
    .unwrap();

    // Run migrations
    info!("Running db migrations");
    sqlx::migrate!().run(&db).await.unwrap();

    // Create entity manager
    // and load current state
    let mut entity_manager = EntityManager::new();
    let mut match_manager = MatchManager::new();

    // Create new match
    // #NOTE: #HACK: during dev, we just create a new isolated match each time
    //               we restart
    let test_match = MatchConfig::isolated(100);
    test_match
        .save(&db)
        .await
        .expect("Failed to save new match config");

    // And prepare it to run
    match_manager
        .initialise_new_match(test_match, &mut entity_manager, &db)
        .await
        .expect("Failed to initialise match");

    // Create channel for tick events
    let (tick_tx, mut tick_rx) = broadcast::channel::<TickEvent>(10);

    // Create service and handle
    let (qubit_service, qubit_handle) = router.into_service(QubitCtx {
        tick_tx: tick_tx.clone(),
    });

    // Nest into an Axum router
    let axum_router = axum::Router::<()>::new().nest_service("/rpc", qubit_service);

    // Setup a task tracker
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

    // Start the tick loop
    tracker.spawn({
        let token = token.clone();

        let start_loop = async move {
            let mut tick_count = 0;
            loop {
                tick_tx
                    .send(TickEvent::StartOfTick {
                        tick_id: tick_count,
                    })
                    .expect("Cannot send start of tick event");

                // Run the next tick
                match_manager
                    .perform_match_tick(&tick_tx, &mut entity_manager, &db)
                    .await;

                // Tell em we finished the tick
                tick_tx
                    .send(TickEvent::EndOfTick {
                        tick_id: tick_count,
                    })
                    .expect("Cannot send end of tick event");

                // Wait for next tick...
                tick_count += 1;
                tokio::time::sleep(TICK_DELAY).await;
            }
        };

        async move {
            tokio::select! {
                () = start_loop => {},
                () = token.cancelled() => {},
            }
        }
    });

    // Generate tracing logs for tick events
    tracker.spawn({
        let token = token.clone();
        let start_loop = async move {
            while let Ok(ev) = tick_rx.recv().await {
                debug!("tick event {ev:?}");
            }
        };

        async move {
            tokio::select! {
                () = start_loop => {},
                () = token.cancelled() => {},
            }
        }
    });

    // Start a Hyper server
    tracker.spawn({
        let token = token.clone();

        info!("RPC server listening at 127.0.0.1:9944");
        let start_hyper = axum::serve(
            TcpListener::bind(&SocketAddr::from(([127, 0, 0, 1], 9944)))
                .await
                .unwrap(),
            axum_router,
        );

        async move {
            tokio::select! {
                err = start_hyper => {
                    // Stop qubit!
                    qubit_handle.stop().expect("Could not stop qubit");
                    err.unwrap();
                },
                () = token.cancelled() => {},
            }
        }
    });

    // Wait for signal...
    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");

    // Then kill everything
    token.cancel();
    tracker.close();
    tracker.wait().await;
}
