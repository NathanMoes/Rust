use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub genres: Vec<String>,
    pub popularity: i32,
    pub followers: i32,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist_ids: Vec<String>,
    pub artist_names: Vec<String>,
    pub album_id: String,
    pub album_name: String,
    pub duration_ms: i32,
    pub popularity: i32,
    pub explicit: bool,
    pub danceability: f64,
    pub energy: f64,
    pub key: i32,
    pub loudness: f64,
    pub mode: i32,
    pub speechiness: f64,
    pub acousticness: f64,
    pub instrumentalness: f64,
    pub liveness: f64,
    pub valence: f64,
    pub tempo: f64,
    pub time_signature: i32,
    pub preview_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist_ids: Vec<String>,
    pub release_date: String,
    pub total_tracks: i32,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyImportRequest {
    pub playlist_id: String,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub track_names: Vec<String>,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub seed_track_ids: Vec<String>,
    pub target_valence: Option<f64>,
    pub target_energy: Option<f64>,
    pub target_danceability: Option<f64>,
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YouTubeVideo {
    pub id: String,
    pub title: String,
    pub channel_title: String,
    pub duration: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedPlaylist {
    pub id: String,
    pub name: String,
    pub url: String,
    pub tracks_added: i32,
    pub tracks_not_found: Vec<String>,
}
