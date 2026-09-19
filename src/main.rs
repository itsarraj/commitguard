use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use commitguard::{changelog, gitlog, hook, lint, parse};

#[derive(Parser)]
#[command(
    name = "commitguard",
    about = "A native Conventional Commits linter + changelog generator"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lint a commit message file (what a `commit-msg` hook passes as $1).
    Lint { message_file: PathBuf },
    /// Install the commit-msg hook into a repo's .git/hooks.
    InstallHook {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Generate a changelog from Conventional Commit history.
    Changelog {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// A git revision range, e.g. `v1.0.0..HEAD`. Omit for full history.
        #[arg(long)]
        range: Option<String>,
        #[arg(long, default_value = "Unreleased")]
        version: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Lint { message_file } => {
            let content = fs::read_to_string(&message_file)
                .map_err(|e| anyhow::anyhow!("reading {}: {e}", message_file.display()))?;
            let issues = lint::lint(&content);
            if issues.is_empty() {
                Ok(())
            } else {
                eprintln!("commitguard: commit message rejected\n");
                for issue in &issues {
                    eprintln!("  [{}] {}", issue.rule, issue.message);
                }
                eprintln!("\nsee https://www.conventionalcommits.org/ for the format expected");
                std::process::exit(1);
            }
        }
        Command::InstallHook { repo } => {
            let path = hook::install_hook(&repo)?;
            println!("installed commit-msg hook at {}", path.display());
            Ok(())
        }
        Command::Changelog {
            repo,
            range,
            version,
        } => {
            let messages = gitlog::commit_messages(&repo, range.as_deref())?;
            let commits: Vec<_> = messages
                .iter()
                .filter_map(|m| parse::parse(m).ok())
                .collect();
            print!("{}", changelog::render_changelog(&commits, &version));
            Ok(())
        }
    }
}
