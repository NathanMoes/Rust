use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub genres: Vec<String>,
    pub popularity: u32,
    pub followers: u32,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artist_ids: Vec<String>,
    pub artist_names: Vec<String>,
    pub album_id: String,
    pub album_name: String,
    pub duration_ms: u32,
    pub popularity: u32,
    pub explicit: bool,
    pub preview_url: Option<String>,
    // Audio features
    pub danceability: f32,
    pub energy: f32,
    pub key: i32,
    pub loudness: f32,
    pub mode: i32,
    pub speechiness: f32,
    pub acousticness: f32,
    pub instrumentalness: f32,
    pub liveness: f32,
    pub valence: f32,
    pub tempo: f32,
    pub time_signature: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YouTubeVideo {
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatedPlaylist {
    pub playlist_id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotifyImportRequest {
    pub playlist_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YouTubePlaylistRequest {
    pub title: String,
    pub description: String,
    pub track_queries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub track_id: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}
