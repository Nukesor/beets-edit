use std::{fs::OpenOptions, path::PathBuf};

use regex::Regex;
use serde::Deserialize;

use crate::{commands::write_document, config::Config, internal_prelude::*, models::Track};

pub fn edit_tracks(config: &Config, path: &PathBuf) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(path)
        .wrap_err_with(|| format!("Failed to open file at {path:?}"))?;

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
                        "Album - found exact match {expr} on track {}",
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
                        "Album - found regex match {expr} on track {}",
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

    write_document(&mut file, tracks)?;

    Ok(())
}
