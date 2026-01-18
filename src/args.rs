use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(name = "beets_edit", author, version)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
pub enum SubCommand {
    /// Entry point
    ///
    /// Run in the artist directory in question.
    Run,

    /// Edit an album's tracks
    ///
    /// This is called by `beet edit` via the `EDITOR` variable.
    EditTracks {
        /// The temporary yaml file to edit
        path: PathBuf,
    },

    /// Edit an album
    ///
    /// This is called by `beet edit -a` via the `EDITOR` variable.
    EditAlbum {
        /// The temporary yaml file to edit
        path: PathBuf,
    },
}
