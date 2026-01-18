use std::{
    env::current_dir,
    fs::read_dir,
    io::Write,
    path::Path,
    process::{Child, Command, ExitStatus, Stdio},
    time::Duration,
};

use color_eyre::eyre::ContextCompat;
use regex::Regex;

use crate::{config::Config, internal_prelude::*};

pub fn run(config: &Config) -> Result<()> {
    let cwd = current_dir().wrap_err("Couldn't determin cwd.")?;
    for entry in read_dir(cwd)? {
        let entry = entry.wrap_err("Couldn't open directory entry.")?;
        let path = entry.path();

        handle_entry(&path, config)?;
    }

    Ok(())
}

/// Handle a single directory
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
