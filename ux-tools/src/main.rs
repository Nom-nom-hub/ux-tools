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

    #[arg(long)]
    offline: bool,

    #[arg(long)]
    use_uv: bool,
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
    Version,
}

#[derive(Subcommand)]
enum CacheCommands {
    Ls,
    Rm {
        tool: String,
    },
    Clean,
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
    let cache_dir = cache.cache_dir();
    let runner = Runner::new(cache_dir.to_path_buf());

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
        },
        Some(Commands::Version) => {
            println!("ux v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            if let Some(tool) = cli.tool {
                let mut parts = tool.splitn(2, '@');
                let (name, _version) = match (parts.next(), parts.next()) {
                    (Some(n), Some(v)) => (n.to_string(), Some(v)),
                    (Some(n), None) => (n.to_string(), None),
                    _ => anyhow::bail!("Invalid tool specification"),
                };

                if cli.offline {
                    if cache.has_cached(&name) {
                        info!("Running {} from cache", name);
                        let exit_code = runner.run_tool(&name, &cli.args).await?;
                        std::process::exit(exit_code);
                    } else {
                        anyhow::bail!("Tool not cached: {}", name);
                    }
                } else if cli.use_uv {
                    let client = PypiClient::new();
                    let (_name, version) = client.get_package(&name).await?;
                    println!("Resolving {} v{}", name, version);
                    let exit_code = runner.run_with_uv(&name, &cli.args).await?;
                    std::process::exit(exit_code);
                } else {
                    let client = PypiClient::new();
                    let (_name, version) = client.get_package(&name).await?;
                    println!("Resolving {} v{}", name, version);
                    println!("Installing...");

                    match runner.warm(&name).await {
                        Ok(info) => println!("Installed {}", info.version),
                        Err(e) => eprintln!("Install failed: {}", e),
                    }

                    let exit_code = runner.run_tool(&name, &cli.args).await?;
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
                println!();
                println!("Options:");
                println!("  --use-uv           # Delegate to uv (faster)");
                println!("  --offline         # Run only from cache");
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