use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// DevPane command line interface.
#[derive(Debug, Parser)]
#[command(name = "devpane")]
#[command(about = "Open and inspect DevPane terminal workspace files.")]
pub struct Cli {
    /// Command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Supported DevPane commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate a `.dpane` file.
    Validate {
        /// Path to the `.dpane` file.
        path: PathBuf,
    },

    /// Print a resolved summary of a `.dpane` file.
    Inspect {
        /// Path to the `.dpane` file.
        path: PathBuf,
    },

    /// Print the process launch plan for auto-start panes.
    Plan {
        /// Path to the `.dpane` file.
        path: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_validate_command() {
        let cli = Cli::parse_from(["devpane", "validate", "examples/webclient.dpane"]);

        match cli.command {
            Command::Validate { path } => {
                assert_eq!(path, PathBuf::from("examples/webclient.dpane"));
            }
            Command::Inspect { .. } | Command::Plan { .. } => panic!("expected validate command"),
        }
    }

    #[test]
    fn parses_inspect_command() {
        let cli = Cli::parse_from(["devpane", "inspect", "examples/webclient.dpane"]);

        match cli.command {
            Command::Inspect { path } => {
                assert_eq!(path, PathBuf::from("examples/webclient.dpane"));
            }
            Command::Validate { .. } | Command::Plan { .. } => panic!("expected inspect command"),
        }
    }

    #[test]
    fn parses_plan_command() {
        let cli = Cli::parse_from(["devpane", "plan", "examples/webclient.dpane"]);

        match cli.command {
            Command::Plan { path } => {
                assert_eq!(path, PathBuf::from("examples/webclient.dpane"));
            }
            Command::Validate { .. } | Command::Inspect { .. } => {
                panic!("expected plan command")
            }
        }
    }
}
