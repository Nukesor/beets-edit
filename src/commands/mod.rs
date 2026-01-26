use std::{
    fs::File,
    io::{Seek, Write},
};

use regex::Regex;
use serde::Serialize;

use crate::{config::Rewrite, internal_prelude::*};

pub mod edit_albums;
pub mod edit_tracks;
pub mod run;

pub fn write_document<T: Serialize>(file: &mut File, items: Vec<T>) -> Result<()> {
    // Concatenate all tracks into a single large multi-document.
    let mut output = String::new();
    let mut tracks = items.iter().peekable();
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

/// Return whether a given string (`hay`) matches any expression in the given [Rewrite].
pub fn rewrite_matches(rewrite: &Rewrite, hay: &str, context: &str, field: &str) -> Result<bool> {
    for expr in &rewrite.expressions {
        // Check for direct matches (no regex)
        if hay == expr {
            info!("{context} - found exact match '{expr}' on {field} '{hay}'",);
            return Ok(true);
        }

        // Check for regex matches (actual regexes).
        let re = Regex::new(expr).wrap_err_with(|| format!("Found invalid expression {expr}"))?;

        if re.is_match(hay) {
            info!("{context} - found regex match '{expr}' on {field} '{hay}'",);
            return Ok(true);
        }
    }
    Ok(false)
}
