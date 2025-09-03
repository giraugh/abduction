mod brain;
mod entity;
mod hex;
mod location;
mod logs;
mod mtch;
mod player_gen;

use futures::{Stream, StreamExt};
use qubit::{handler, Router};
use sqlx::{sqlite::SqliteConnectOptions, Pool, Sqlite, SqlitePool};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::fs;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::entity::Entity;
use crate::logs::GameLog;
use crate::mtch::{MatchConfig, MatchManager, TickEvent};

const TICK_DELAY: Duration = Duration::from_millis(500);
const MATCH_COOLDOWN_DURATION: Duration = Duration::from_secs(1_200); // 20mins

pub type Db = Pool<Sqlite>;

/// The context type for qubit
#[derive(Clone)]
struct QubitCtx {
    /// Sender for tick events
    /// (This is lifecycle events and entity updates)
    tick_tx: broadcast::Sender<TickEvent>,

    /// Sender for game logs
    /// (This flavour and system events shown to users)
    log_tx: broadcast::Sender<GameLog>,

    /// Db pool
    db: Db,

    /// When a match is running,
    /// the match manager for that match
    match_manager: Arc<Mutex<Option<MatchManager>>>,
}

/// Get the current state of all entities
#[handler(query)]
async fn get_entity_states(ctx: QubitCtx) -> Option<Vec<Entity>> {
    ctx.match_manager
        .lock()
        .await
        .as_ref()
        .map(MatchManager::all_entity_states)
}

/// Get the config for the current match
/// Returns null if no current match
#[handler(query)]
async fn get_match_config(ctx: QubitCtx) -> Option<MatchConfig> {
    ctx.match_manager
        .lock()
        .await
        .as_ref()
        .map(|mm| mm.match_config.clone())
}

/// Get a stream of all tick events
#[handler(subscription)]
async fn events_stream(ctx: QubitCtx) -> impl Stream<Item = TickEvent> {
    let stream = tokio_stream::wrappers::BroadcastStream::new(ctx.tick_tx.subscribe());
    stream.filter_map(|e| async { e.ok() })
}

/// Get a stream of game logs
/// TODO: these should prob be saved to the DB too
#[handler(subscription)]
async fn game_log_stream(ctx: QubitCtx) -> impl Stream<Item = GameLog> {
    let stream = tokio_stream::wrappers::BroadcastStream::new(ctx.log_tx.subscribe());
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
    let router = Router::new()
        .handler(get_entity_states)
        .handler(get_match_config)
        .handler(game_log_stream)
        .handler(events_stream);

    // Generate ts types
    if fs::try_exists("../abduction-site").await.unwrap() {
        info!("Writing ts bindings");
        router
            .generate_type("../abduction-site/src/lib/api.gen.ts")
            .expect("Failed to write bindings");
    } else {
        warn!("Skipping writing ts bindings");
    }

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

    // Create channel for tick events
    let (tick_tx, mut tick_rx) = broadcast::channel::<TickEvent>(10);

    // Create channel for game logs
    let (log_tx, mut log_rx) = broadcast::channel::<GameLog>(10);

    // Create a spot that could later be a match manager (youll see)
    let match_manager = Arc::default();
    let qubit_ctx = QubitCtx {
        tick_tx: tick_tx.clone(),
        log_tx: log_tx.clone(),
        db: db.clone(),
        match_manager,
    };

    // Create service and handle
    let (qubit_service, qubit_handle) = router.into_service(qubit_ctx.clone());

    // Nest into an Axum router
    let axum_router = axum::Router::<()>::new().nest_service("/rpc", qubit_service);

    // Setup a task tracker
    let tracker = TaskTracker::new();
    let token = CancellationToken::new();

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

    // Generate tracing logs for log events
    tracker.spawn({
        let token = token.clone();
        let start_loop = async move {
            while let Ok(ev) = log_rx.recv().await {
                debug!("{ev:?}");
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

        info!("RPC server listening at 0.0.0.0:9944");
        let start_hyper = axum::serve(
            TcpListener::bind(&SocketAddr::from(([0, 0, 0, 0], 9944)))
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

    // Go check if we need to be running a match now
    // and/or load the schedule for the next one
    tracker.spawn({
        let token = token.clone();
        let start_match_runner = async move {
            loop {
                // Run the match...
                // if it returns its because the match ended, if so just loop it back
                run_match_now(qubit_ctx.clone()).await.unwrap();
            }
        };

        async move {
            tokio::select! {
                () = start_match_runner => {},
                () = token.cancelled() => {},
            }
        }
    });

    // Wait for shutdown signal...
    tokio::signal::ctrl_c().await.unwrap();
    info!("Shutting down...");

    // Then kill everything
    token.cancel();
    tracker.close();
    tracker.wait().await;
}

async fn run_match_now(ctx: QubitCtx) -> anyhow::Result<()> {
    // Is there an incomplete one to keep running?
    let match_manager = match MatchConfig::get_incomplete(&ctx.db).await? {
        // If so then just load it now
        Some(match_config) => {
            info!("Loading in-progress match ({})", match_config.match_id);
            MatchManager::load_match(match_config, &ctx.db).await
        }

        // Otherwise, consult the shedule and possibly wait till later
        // or, create a new match, save it to the db and initialise entities for it
        None => {
            // Are we supposed to be running yet?
            // TODO:
            // info!("Checking match schedule");
            // TODO

            // TODO: actually check schedule but for now just wait for a delay
            sleep(MATCH_COOLDOWN_DURATION).await;

            // Okay cool, create a new match
            info!("Creating a new match");
            let dev_match = MatchConfig::isolated(15, 10);
            dev_match
                .save(&ctx.db)
                .await
                .expect("Failed to save new match config");

            // Create match manager
            // and prepare it to run
            let mut match_manager = MatchManager::load_match(dev_match, &ctx.db).await;
            match_manager
                .initialise_new_match(&ctx.db)
                .await
                .expect("Failed to initialise match");

            // Fire off a "new match started" event
            ctx.tick_tx.send(TickEvent::StartOfMatch)?;

            match_manager
        }
    };

    // Update the shared match manager to this new match manager w/ the loaded match
    {
        let mut shared_match_manager = ctx.match_manager.lock().await;
        *shared_match_manager = Some(match_manager);
    }

    // Start the tick loop
    tick_loop(ctx).await
}

async fn tick_loop(ctx: QubitCtx) -> anyhow::Result<()> {
    // Start the tick loop
    info!("Starting main tick loop");
    let mut tick_count = 0;
    loop {
        ctx.tick_tx
            .send(TickEvent::StartOfTick {
                tick_id: tick_count,
            })
            .expect("Cannot send start of tick event");

        // Generate updates for this tick
        ctx.match_manager
            .lock()
            .await
            .as_mut()
            .expect("Tick loop is running but match manager isnt present...")
            .perform_match_tick(&ctx)
            .await;

        // Tell em we finished the tick
        ctx.tick_tx
            .send(TickEvent::EndOfTick {
                tick_id: tick_count,
            })
            .expect("Cannot send end of tick event");

        // Did the match just finish?
        {
            let mut maybe_mm = ctx.match_manager.lock().await;
            let mm = maybe_mm
                .as_mut()
                .expect("Tick loop is running but match_manager isn't present...");
            if mm.match_over() {
                info!("Match completed");

                // Update the config to set `complete=true`
                mm.match_config.complete = true;
                mm.match_config.save(&ctx.db).await?;

                // Send an event
                ctx.tick_tx.send(TickEvent::EndOfMatch)?;

                // Remove the shared manager
                *maybe_mm = None;

                // Break the loop
                break;
            }
        }

        // Wait for next tick...
        tick_count += 1;
        tokio::time::sleep(TICK_DELAY).await;
    }

    Ok(())
}
