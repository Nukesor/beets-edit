use std::{
    fs::OpenOptions,
    io::{Seek, Write},
    path::PathBuf,
};

use regex::Regex;
use serde::Deserialize;

use crate::{config::Config, internal_prelude::*, models::Album};

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
