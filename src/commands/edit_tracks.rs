use std::{fs::OpenOptions, path::PathBuf};

use serde::Deserialize;

use crate::{
    commands::{rewrite_matches, write_document},
    config::Config,
    internal_prelude::*,
    models::Track,
};

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

        let mut found_rewrite = None;

        for artist_rewrite in &config.artist_rewrites {
            if rewrite_matches(artist_rewrite, &track.artist, "Track", "artist")? {
                found_rewrite = Some(artist_rewrite.clone());
                break;
            }
        }

        if let Some(rewrite) = found_rewrite {
            if let Some(single) = rewrite.single {
                track.artist = single;
            }

            if let Some(multi) = rewrite.multi {
                track.artists = multi.join("\\␀");
            }
        }

        let mut found_rewrite = None;

        for albumartist_rewrite in &config.albumartist_rewrites {
            if rewrite_matches(
                albumartist_rewrite,
                &track.albumartist,
                "Track",
                "albumartist",
            )? {
                found_rewrite = Some(albumartist_rewrite.clone());
                break;
            }
        }

        if let Some(rewrite) = found_rewrite {
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
