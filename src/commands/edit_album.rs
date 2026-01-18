use std::{fs::OpenOptions, path::PathBuf};

use serde::Deserialize;

use crate::{
    commands::{rewrite_matches, write_document},
    config::Config,
    internal_prelude::*,
    models::Album,
};

pub fn edit_album(config: &Config, path: &PathBuf) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .open(path)
        .wrap_err_with(|| format!("Failed to open file at {path:?}"))?;

    let mut albums = Vec::new();
    for album in serde_yaml::Deserializer::from_reader(&file) {
        let mut album = Album::deserialize(album)?;
        debug!("{:?}", album);

        let mut found_rewrite = None;

        for albumartist_rewrite in &config.albumartist_rewrites {
            if rewrite_matches(
                albumartist_rewrite,
                &album.albumartist,
                "Album",
                "albumartist",
            )? {
                found_rewrite = Some(albumartist_rewrite.clone());
                break;
            }
        }

        if let Some(rewrite) = found_rewrite
            && let Some(single) = rewrite.single
        {
            album.albumartist = single;
        }

        albums.push(album);
    }

    write_document(&mut file, albums)?;

    Ok(())
}
