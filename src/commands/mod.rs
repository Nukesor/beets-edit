use std::{
    fs::File,
    io::{Seek, Write},
};

use serde::Serialize;

use crate::internal_prelude::*;

pub mod edit_album;
pub mod edit_track;
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
