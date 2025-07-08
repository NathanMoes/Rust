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
pub struct BackendYouTubeVideo {
    pub id: String,
    pub title: String,
    pub channel_title: String,
    pub duration: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackWithYouTube {
    pub track: Track,
    pub youtube_video: Option<BackendYouTubeVideo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimilarTracksResponse {
    pub original_track: Track,
    pub similar_tracks: Vec<TrackWithYouTube>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "track", "artist", "album"
    pub properties: GraphNodeProperties,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeProperties {
    pub name: String,
    pub popularity: Option<i32>,
    pub genres: Option<Vec<String>>,
    pub audio_features: Option<AudioFeatures>,
    pub image_url: Option<String>,
    pub duration_ms: Option<i32>,
    pub artist_names: Option<Vec<String>>,
    pub album_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioFeatures {
    pub danceability: f64,
    pub energy: f64,
    pub valence: f64,
    pub tempo: f64,
    pub acousticness: f64,
    pub instrumentalness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String, // "PERFORMED", "CONTAINS"
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
