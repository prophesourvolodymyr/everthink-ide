use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "everthink",
    about = "Everthink IDE — AI coding assistant",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project from an idea
    Init {
        /// The project idea or name
        idea: String,
    },
    /// Add a new feature (runs AUDIT phase)
    Add {
        /// The feature to add
        feature: String,
    },
    /// Build current feature or all pending tasks
    Build {
        /// Build all pending tasks
        #[arg(long)]
        all: bool,
        /// Run without confirmation prompts (YOLO mode)
        #[arg(long)]
        yolo: bool,
    },
    /// Resume the last session
    Continue,
    /// Run cargo clippy + build + test
    Verify,
    /// Search the codebase with Greppy
    Search {
        /// The search query
        query: String,
    },
    /// Load a topic from memory
    Remember {
        /// The topic to recall
        topic: String,
    },
}

pub async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand → launch the TUI
            crate::tui::run().await?;
        }
        Some(Commands::Init { idea }) => {
            eprintln!("[stub] init: {idea}");
        }
        Some(Commands::Add { feature }) => {
            eprintln!("[stub] add: {feature}");
        }
        Some(Commands::Build { all, yolo }) => {
            let config = crate::config::Config::load().unwrap_or_default();
            let registry = crate::providers::ProviderRegistry::new(&config);
            let provider = registry.default_provider();

            if all {
                // Full autonomous build: read pending tasks → LLM → verify → mark done
                let auto_config = crate::core::autonomous::AutonomousConfig {
                    yolo,
                    ..Default::default()
                };
                let build =
                    crate::core::autonomous::AutonomousBuild::new(auto_config, provider);
                let (out_tx, mut out_rx) =
                    tokio::sync::mpsc::unbounded_channel::<String>();

                tokio::spawn(async move {
                    if let Err(e) = build.run_all(out_tx).await {
                        eprintln!("[build] error: {e}");
                    }
                });

                // Forward output to stdout
                while let Some(line) = out_rx.recv().await {
                    print!("{line}");
                }
            } else {
                // No --all: just list pending phases
                let pm = crate::core::progress::ProgressManager::from_cwd();
                match pm.pending_tasks() {
                    Ok(pending) if pending.is_empty() => {
                        println!("All phases complete — nothing pending.");
                    }
                    Ok(pending) => {
                        println!("{} pending phase(s):", pending.len());
                        for task in &pending {
                            println!("  Phase {} — {}", task.phase, task.name);
                        }
                        println!();
                        if yolo {
                            println!("Run `everthink build --all --yolo` to build without prompts.");
                        } else {
                            println!("Run `everthink build --all` to start autonomous build.");
                            println!("Add `--yolo` to skip confirmation prompts.");
                        }
                    }
                    Err(e) => {
                        eprintln!("[build] Could not read PROGRESS.md: {e}");
                    }
                }
            }
        }
        Some(Commands::Continue) => {
            let store = crate::storage::SessionStore::from_cwd();
            match store.load_latest() {
                Ok(Some(session)) => {
                    eprintln!(
                        "[everthink] Restoring session {} ({} messages)",
                        session.id,
                        session.messages.len()
                    );
                    crate::tui::run_with_session(session).await?;
                }
                Ok(None) => {
                    eprintln!("[everthink] No previous session found. Starting fresh.");
                    crate::tui::run().await?;
                }
                Err(e) => {
                    eprintln!("[everthink] Failed to load session: {e}. Starting fresh.");
                    crate::tui::run().await?;
                }
            }
        }
        Some(Commands::Verify) => {
            eprintln!("[stub] verify");
        }
        Some(Commands::Search { query }) => {
            eprintln!("[stub] search: {query}");
        }
        Some(Commands::Remember { topic }) => {
            eprintln!("[stub] remember: {topic}");
        }
    }

    Ok(())
}
