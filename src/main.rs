use clap::Parser;
use internal_prelude::*;

use crate::{
    args::CliArguments,
    commands::{edit_albums::edit_album, edit_tracks::edit_tracks, run::run},
    config::Config,
};

pub mod args;
pub mod commands;
pub mod config;
pub mod models;
pub mod tracing;

pub(crate) mod internal_prelude {
    #[allow(unused_imports)]
    pub(crate) use tracing::{debug, error, info, trace, warn};

    pub(crate) use crate::errors::*;
}

pub(crate) mod errors {
    pub use color_eyre::Result;
    #[allow(unused_imports)]
    pub use color_eyre::eyre::{ContextCompat, WrapErr, bail, eyre};
}

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Set the verbosity level of the logger.
    tracing::install_tracing(opt.verbose)?;
    color_eyre::install()?;

    let config = Config::read()?;

    match opt.cmd {
        args::SubCommand::Run => run(&config),
        args::SubCommand::EditTracks { path } => edit_tracks(&config, &path),
        args::SubCommand::EditAlbums { path } => edit_album(&config, &path),
    }
}
