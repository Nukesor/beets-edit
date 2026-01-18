use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Track {
    pub album: String,
    pub albumartist: String,
    pub albumartists: String,
    pub artist: String,
    pub artists: String,
    pub id: usize,
    pub title: String,
    pub track: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Album {
    pub album: String,
    pub albumartist: String,
    pub id: usize,
}
