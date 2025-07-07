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
    pub playlist_url: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackWithYouTube {
    pub track: Track,
    pub youtube_video: Option<YouTubeVideo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarTracksResponse {
    pub original_track: Track,
    pub similar_tracks: Vec<TrackWithYouTube>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "track", "artist", "album"
    pub properties: GraphNodeProperties,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioFeatures {
    pub danceability: f64,
    pub energy: f64,
    pub valence: f64,
    pub tempo: f64,
    pub acousticness: f64,
    pub instrumentalness: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String, // "PERFORMED", "CONTAINS"
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphSearchRequest {
    pub query: Option<String>,
    pub limit: Option<i32>,
    pub node_types: Option<Vec<String>>,
}
