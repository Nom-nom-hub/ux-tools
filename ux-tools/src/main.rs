use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;

mod cache;
mod pypi;
mod runner;
mod sources;

use cache::ToolCache;
use pypi::PypiClient;
use runner::Runner;

#[derive(Parser)]
#[command(name = "ux")]
#[command(about = "A faster, smarter Python tool runner", long_about = None)]
struct Cli {
    tool: Option<String>,

    #[arg(last = true, allow_hyphen_values = true)]
    args: Vec<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true, short, long)]
    verbose: bool,

    #[arg(long, global = true)]
    offline: bool,

    #[arg(long, global = true)]
    use_uv: bool,

    #[arg(long, global = true, short = 'u')]
    update: bool,

    #[arg(long, global = true, short = 'c')]
    check_updates: bool,
}

#[derive(Subcommand)]
enum Commands {
    Warm {
        tool: Option<String>,

        #[arg(long)]
        all: bool,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },
    Update {
        tool: Option<String>,

        #[arg(long)]
        all: bool,

        #[arg(long, short = 'y')]
        yes: bool,
    },
    Version,
}

#[derive(Subcommand)]
enum CacheCommands {
    Ls,
    Rm {
        tool: String,
    },
    Clean,
    Prune,
    Stats,
}

#[derive(Subcommand)]
enum AliasCommands {
    Add {
        alias: String,

        tool: String,
    },
    Rm {
        alias: String,
    },
    Ls,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let mut cache = ToolCache::new()?;
    let cache_dir = cache.cache_dir().to_path_buf();
    let runner = Runner::new(cache_dir);

    match cli.command {
        Some(Commands::Warm { tool, all }) => {
            if all {
                info!("Warming all cached tools");
                for entry in cache.list_entries() {
                    println!("Warming: {}...", entry.tool);
                    match runner.warm(&entry.tool).await {
                        Ok(info) => println!("  ✓ {} ({})", info.tool, info.version),
                        Err(e) => eprintln!("  ✗ {}: {}", entry.tool, e),
                    }
                }
                println!("Done");
            } else if let Some(t) = tool {
                info!("Warming tool: {}", t);
                println!("Warming: {}...", t);
                let info = runner.warm(&t).await?;
                println!("Warmed: {} ({})", info.tool, info.version);
            } else {
                anyhow::bail!("Specify a tool or --all");
            }
        }
        Some(Commands::Cache { command }) => match command {
            CacheCommands::Ls => {
                let entries = cache.list_entries();
                if entries.is_empty() {
                    println!("No cached tools");
                } else {
                    println!("Cached tools:");
                    for entry in entries {
                        println!(
                            "  {} v{} ({})",
                            entry.tool, entry.version, entry.python_version
                        );
                    }
                }
            }
            CacheCommands::Rm { tool } => {
                if let Some(_) = cache.remove_entry(&tool)? {
                    let venv_path = cache.cache_dir().join("venvs").join(&tool);
                    if venv_path.exists() {
                        tokio::fs::remove_dir_all(venv_path).await?;
                    }
                    println!("Removed: {}", tool);
                } else {
                    println!("Tool not cached: {}", tool);
                }
            }
            CacheCommands::Clean => {
                cache.clear()?;
                println!("Cache cleaned");
            }
            CacheCommands::Prune => {
                let removed = cache.prune_old()?;
                println!("Removed {} old entries", removed);
            }
            CacheCommands::Stats => {
                cache.print_stats()?;
            }
        },
        Some(Commands::Alias { command }) => match command {
            AliasCommands::Add { alias, tool } => {
                cache.add_alias(&alias, &tool)?;
                println!("Added alias: {} -> {}", alias, tool);
            }
            AliasCommands::Rm { alias } => {
                cache.remove_alias(&alias)?;
                println!("Removed alias: {}", alias);
            }
            AliasCommands::Ls => {
                cache.list_aliases();
            }
        },
        Some(Commands::Update { tool, all, yes }) => {
            if all {
                info!("Updating all tools");
                let entries = cache.list_entries();
                for entry in entries {
                    print!("Updating {}... ", entry.tool);
                    match runner.update(&entry.tool).await {
                        Ok(v) => println!("updated to {}", v),
                        Err(e) => println!("failed: {}", e),
                    }
                }
                println!("Done");
            } else if let Some(t) = tool {
                info!("Updating tool: {}", t);
                println!("Updating: {}...", t);
                let new_version = runner.update(&t).await?;
                println!("Updated to {}", new_version);
            } else {
                anyhow::bail!("Specify a tool or --all");
            }
        },
        Some(Commands::Version) => {
            println!("ux v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            if cli.check_updates {
                let updates = runner.check_all_updates().await?;
                if updates.is_empty() {
                    println!("All tools up to date");
                } else {
                    println!("Updates available:");
                    for (tool, old_v, new_v) in updates {
                        println!("  {}: {} -> {}", tool, old_v, new_v);
                    }
                }
                return Ok(());
            }

            if let Some(tool) = cli.tool {
                let resolved = tool.as_str();

                if cli.offline {
                    if cache.has_cached(&resolved) {
                        info!("Running {} from cache", resolved);
                        let exit_code = runner.run_tool(&resolved, &cli.args).await?;
                        std::process::exit(exit_code);
                    } else {
                        anyhow::bail!("Tool not cached: {}", resolved);
                    }
                } else if cli.use_uv {
                    let client = PypiClient::new();
                    let (_name, version) = client.get_package(&resolved).await?;
                    println!("Resolving {} v{}", resolved, version);
                    let exit_code = runner.run_with_uv(&resolved, &cli.args).await?;
                    std::process::exit(exit_code);
                } else {
                    let client = PypiClient::new();
                    let (_name, version) = client.get_package(&resolved).await?;
                    println!("Resolving {} v{}", resolved, version);
                    println!("Installing...");

                    match runner.warm(&resolved).await {
                        Ok(info) => println!("Installed {}", info.version),
                        Err(e) => eprintln!("Install failed: {}", e),
                    }

                    let exit_code = runner.run_tool(&resolved, &cli.args).await?;
                    std::process::exit(exit_code);
                }
            } else {
                println!("ux v{} - A faster, smarter Python tool runner", env!("CARGO_PKG_VERSION"));
                println!();
                println!("Usage:");
                println!("  ux <tool> [args...]     # Run a tool");
                println!("  ux warm <tool>       # Pre-warm a tool's environment");
                println!("  ux warm --all       # Pre-warm all cached tools");
                println!("  ux cache ls         # List cached tools");
                println!("  ux cache rm <tool> # Remove a cached tool");
                println!("  ux --check-updates # Check for updates");
                println!();
                println!("Options:");
                println!("  --use-uv           # Delegate to uv (faster)");
                println!("  --offline         # Run only from cache");
                println!("  --update, -u      # Auto-update tools");
                println!("  --check-updates, -c # Check for available updates");
                println!();
                println!("Examples:");
                println!("  ux ruff .");
                println!("  ux black --check main.py");
                println!("  ux httpie GET https://example.com");
            }
        }
    }

    Ok(())
}