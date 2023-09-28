use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Prints all problems and statistics
    PrintProblems {
        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },

    /// manipulate with data
    Data {
        #[command(subcommand)]
        data_commands: Option<DataCommands>,
    },

    /// Prints information about fmri and what packages need fmri.
    CheckFMRI {
        /// checking valid fmri
        fmri: String,

        /// path to oi userland repo (absolute path)
        #[arg(long, value_name = "PATH")]
        repo_path: PathBuf,

        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },
}

#[derive(Subcommand)]
pub enum DataCommands {
    /// update catalogs
    UpdateAssets {
        /// path to catalog.dependency.C
        #[arg(short, long, value_name = "FILE")]
        catalog: PathBuf,

        /// path to catalog.encumbered.dependency.C
        #[arg(short, long, value_name = "FILE")]
        encumbered_catalog: PathBuf,

        /// path to oi userland repo
        #[arg(long, value_name = "PATH")]
        repo_path: PathBuf,
    },

    /// Run analyze
    Run {
        /// load catalogs (absolute paths)
        #[arg(long, value_name = "FILE")]
        catalog: Vec<PathBuf>,

        /// path to oi userland repo (absolute path)
        #[arg(long, value_name = "PATH")]
        repo_path: PathBuf,

        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },
}