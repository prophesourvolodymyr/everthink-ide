mod cli;
mod tui;
mod core;
mod providers;
mod tools;
mod skills;
mod storage;
mod commands;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
