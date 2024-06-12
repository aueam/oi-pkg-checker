use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    /// Run analyze
    Run {
        /// load catalogs (absolute paths)
        #[arg(long, value_name = "FILE")]
        catalog: Vec<PathBuf>,

        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },

    /// Prints information about fmri and what packages need fmri.
    CheckFMRI {
        /// checking valid fmri
        fmri: String,

        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },
}
