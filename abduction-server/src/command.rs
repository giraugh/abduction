//! Provide commands (just via stdin for now) to administrate the game

use std::{str::FromStr, sync::atomic};

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::info;

use crate::QubitCtx;

#[derive(Debug, Clone, strum::AsRefStr, strum::EnumString)]
pub enum Command {
    #[strum(serialize = "end match", serialize = "end")]
    EndMatch,
}

impl Command {
    pub async fn handle(&self, ctx: QubitCtx) -> anyhow::Result<()> {
        match self {
            Command::EndMatch => {
                info!("Match will end after next tick");
                ctx.flags
                    .force_end_match
                    .store(true, atomic::Ordering::Relaxed);
            }
        }

        Ok(())
    }
}

pub async fn process_stdin_commands(ctx: QubitCtx) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        match Command::from_str(&line) {
            Ok(command) => {
                command.handle(ctx.clone()).await?;
            }
            Err(_) => {
                eprintln!("No such command");
            }
        }
    }

    Ok(())
}
