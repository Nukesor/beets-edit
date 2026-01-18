use std::{
    env::current_dir,
    fs::{OpenOptions, read_dir},
    io::{Seek, Write},
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};

use clap::Parser;
use color_eyre::eyre::ContextCompat;
use internal_prelude::*;
use regex::Regex;
use serde::Deserialize;

use crate::{
    args::CliArguments,
    config::Config,
    models::{Album, Track},
};

pub mod args;
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
    pub use color_eyre::eyre::{WrapErr, bail, eyre};
}

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Set the verbosity level of the logger.
    tracing::install_tracing(opt.verbose)?;
    color_eyre::install()?;

    let config = Config::read()?;

    match opt.cmd {
        args::SubCommand::Run => {
            let cwd = current_dir().wrap_err("Couldn't determin cwd.")?;
            for entry in read_dir(cwd)? {
                let entry = entry.wrap_err("Couldn't open directory entry.")?;
                let path = entry.path();

                handle_entry(&path, &config)?;
            }
            Ok(())
        }
        args::SubCommand::EditTracks { file } => {
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .open(&file)
                .wrap_err_with(|| format!("Failed to open file at {file:?}"))?;

            let mut tracks = Vec::new();
            for track in serde_yaml::Deserializer::from_reader(&file) {
                let mut track = Track::deserialize(track)?;
                debug!("{:?}", track);

                let mut found_artist_rewrite = None;

                'rewrites: for artist_rewrite in &config.artist_rewrites {
                    for expr in &artist_rewrite.expressions {
                        // Check for direct matches (no regex)
                        if track.artist == *expr {
                            info!("Track - found exact match {expr} on track {}", track.artist);
                            found_artist_rewrite = Some(artist_rewrite.clone());
                            break 'rewrites;
                        }

                        // Check for regex matches (actual regexes).
                        let re = Regex::new(expr)
                            .wrap_err_with(|| format!("Found invalid expression {expr}"))?;

                        if re.is_match(&track.albumartist) {
                            info!(
                                "Track - found regex match {expr} on track {}",
                                track.albumartist
                            );

                            found_artist_rewrite = Some(artist_rewrite.clone());
                            break 'rewrites;
                        }
                    }
                }

                if let Some(rewrite) = found_artist_rewrite {
                    if let Some(single) = rewrite.single {
                        track.artist = single;
                    }

                    if let Some(multi) = rewrite.multi {
                        track.artists = multi.join("\\␀");
                    }
                }

                let mut found_album_rewrite = None;

                'rewrites: for album_rewrite in &config.albumartist_rewrites {
                    for expr in &album_rewrite.expressions {
                        // Check for direct matches (no regex)
                        if track.albumartist == *expr {
                            info!(
                                "Track - found exact match {expr} on track {}",
                                track.albumartist
                            );
                            found_album_rewrite = Some(album_rewrite.clone());
                            break 'rewrites;
                        }

                        // Check for regex matches (actual regexes).
                        let re = Regex::new(expr)
                            .wrap_err_with(|| format!("Found invalid expression {expr}"))?;

                        if re.is_match(&track.albumartist) {
                            info!(
                                "Track - found regex match {expr} on track {}",
                                track.albumartist
                            );

                            found_album_rewrite = Some(album_rewrite.clone());
                            break 'rewrites;
                        }
                    }
                }

                if let Some(rewrite) = found_album_rewrite {
                    if let Some(single) = rewrite.single {
                        track.albumartist = single;
                    }

                    if let Some(multi) = rewrite.multi {
                        track.albumartists = multi.join("\\␀");
                    }
                }

                tracks.push(track);
            }

            // Concatenate all tracks into a single large multi-document.
            let mut output = String::new();
            let mut tracks = tracks.iter().peekable();
            while let Some(track) = tracks.next() {
                let serialized = serde_yaml::to_string(&track)?;
                output.push_str(&serialized);

                // Push the yaml document delimiter string, if there's yet another track.
                if tracks.peek().is_some() {
                    output.push_str("\n---\n");
                }
            }

            // Reset the cursor.
            file.rewind().wrap_err("Failed to rewind file?")?;
            // Clear the file content.
            file.set_len(0)
                .wrap_err("Failed to set temporary file len to 0")?;
            file.write_all(output.as_bytes())
                .wrap_err("Failed to write new yaml to temporary file?")?;

            Ok(())
        }
        args::SubCommand::EditAlbum { file } => {
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .open(&file)
                .wrap_err_with(|| format!("Failed to open file at {file:?}"))?;

            let mut albums = Vec::new();
            for album in serde_yaml::Deserializer::from_reader(&file) {
                let mut album = Album::deserialize(album)?;
                debug!("{:?}", album);

                let mut found_album_rewrite = None;

                'rewrites: for album_rewrite in &config.albumartist_rewrites {
                    for expr in &album_rewrite.expressions {
                        // Check for direct matches (no regex)
                        if album.albumartist == *expr {
                            info!(
                                "Track - found exact match {expr} on track {}",
                                album.albumartist
                            );
                            found_album_rewrite = Some(album_rewrite.clone());
                            break 'rewrites;
                        }

                        // Check for regex matches (actual regexes).
                        let re = Regex::new(expr)
                            .wrap_err_with(|| format!("Found invalid expression {expr}"))?;

                        if re.is_match(&album.albumartist) {
                            info!(
                                "Track - found regex match {expr} on track {}",
                                album.albumartist
                            );

                            found_album_rewrite = Some(album_rewrite.clone());
                            break 'rewrites;
                        }
                    }
                }

                if let Some(rewrite) = found_album_rewrite
                    && let Some(single) = rewrite.single
                {
                    album.albumartist = single;
                }

                albums.push(album);
            }

            // Concatenate all tracks into a single large multi-document.
            let mut output = String::new();
            let mut albums = albums.iter().peekable();
            while let Some(track) = albums.next() {
                let serialized = serde_yaml::to_string(&track)?;
                output.push_str(&serialized);

                // Push the yaml document delimiter string, if there's yet another track.
                if albums.peek().is_some() {
                    output.push_str("\n---\n");
                }
            }

            // Reset the cursor.
            file.rewind().wrap_err("Failed to rewind file?")?;
            // Clear the file content.
            file.set_len(0)
                .wrap_err("Failed to set temporary file len to 0")?;
            file.write_all(output.as_bytes())
                .wrap_err("Failed to write new yaml to temporary file?")?;

            Ok(())
        }
    }
}

fn handle_entry(path: &Path, config: &Config) -> Result<()> {
    let file_name = path.file_name().wrap_err("Failed to get filename")?;
    let file_name = file_name.to_string_lossy().to_string();

    for album in &config.albumartist_rewrites {
        for expr in &album.expressions {
            trace!("Check {expr} against {file_name}");
            // Check for direct matches (no regex)
            if file_name == *expr {
                info!("Found exact match {expr} on file {file_name}");
                handle_match(&file_name)?;
                continue;
            }

            // Check for regex matches (actual regexes).
            let re =
                Regex::new(expr).wrap_err_with(|| format!("Found invalid expression {expr}"))?;

            if re.is_match(&file_name) {
                info!("Found match {expr} on file {file_name}");
                handle_match(&file_name)?;
            }
        }
    }

    Ok(())
}

/// Run `beet edit` for the tracks and the album on a matching album artist folder.
///
/// Calls `beets_edit edit-tracks` and `beets_edit edit-album` as `EDITOR` subprocesses.
///
/// That way, we use `beet edit` as a way to interface with beets and our own script as automated
/// way of setting values.
fn handle_match(file_name: &str) -> Result<()> {
    // For the beets query to work, we have to add a trailing `/` so that the directory is matched.
    let mut file_name = file_name.to_string();
    file_name.push('/');

    let mut child = Command::new("beet")
        .arg("edit")
        .arg(&file_name)
        .env("EDITOR", "beets_edit -vvv edit-tracks")
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .wrap_err("failed to execute edit_tracks subprocess")?;

    let status = confirm_edit(&mut child).wrap_err("Failed to confirm edit")?;

    println!("{status:?}");
    if !status.success() {
        return Err(eyre!("Failed to edit album tracks"));
    }

    let mut child = Command::new("beet")
        .arg("edit")
        .arg("-a")
        .arg(file_name)
        .env("EDITOR", "beets_edit -vvv edit-album")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .wrap_err("failed to execute edit_album subprocess")?;

    let status = confirm_edit(&mut child)?;

    if !status.success() {
        return Err(eyre!("Failed to edit album"));
    }

    Ok(())
}

/// Continuously send `A\n` to the subprocess to accept the edits.
fn confirm_edit(child: &mut Child) -> Result<ExitStatus> {
    let mut stdin = child
        .stdin
        .take()
        .wrap_err("Failed to open subprocess stdin")?;
    stdin
        .write_all(b"A\n")
        .wrap_err("Failed to send A\\n to child stdin.")?;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                debug!("Waiting for subprocess to finish");
                std::thread::sleep(Duration::from_millis(500));
            }
            Err(e) => return Err(eyre!("error attempting to wait for child: {e}")),
        }
    }
}
