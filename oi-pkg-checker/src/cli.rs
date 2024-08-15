use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// path to data.bin
    #[arg(long, value_name = "FILE")]
    pub data: PathBuf,

    /// path to oi-userland/components
    #[arg(long, value_name = "FILE")]
    pub components: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Prints all problems and statistics
    PrintProblems,

    /// Run analyze
    Run {
        /// catalog path
        #[arg(long, value_name = "FILE")]
        catalog: Vec<PathBuf>,

        /// set debug on
        #[arg(short, long, default_value = "false")]
        debug: bool,
    },

    /// Prints information about fmri and what packages need that fmri.
    CheckFMRI {
        /// (valid) fmri
        fmri: String,

        /// hide renamed packages
        #[arg(long, default_value = "false")]
        hide_renamed: bool,

        /// print lists in human-readable format
        #[arg(short, long, default_value = "false")]
        human_readable: bool,
    },
}
