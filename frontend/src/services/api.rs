use crate::types::*;
use gloo_net::http::Request;
use serde_json::Value;

const API_BASE_URL: &str = "http://localhost:3000/api";

pub struct ApiService;

impl ApiService {
    pub async fn health_check() -> Result<String, String> {
        let response = Request::get(&format!("{}/health", API_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .text()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))
        } else {
            Err(format!("Server returned status: {}", response.status()))
        }
    }

    pub async fn import_spotify_data(playlist_url: String) -> Result<String, String> {
        let request_body = SpotifyImportRequest { playlist_url };
        
        let response = Request::post(&format!("{}/spotify/import", API_BASE_URL))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .text()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Import failed: {}", error_text))
        }
    }

    pub async fn get_artists() -> Result<Vec<Artist>, String> {
        let response = Request::get(&format!("{}/spotify/artists", API_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Artist>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("Failed to fetch artists: {}", response.status()))
        }
    }

    pub async fn get_tracks() -> Result<Vec<Track>, String> {
        let response = Request::get(&format!("{}/spotify/tracks", API_BASE_URL))
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Track>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("Failed to fetch tracks: {}", response.status()))
        }
    }

    pub async fn get_recommendations(track_id: String, limit: Option<u32>) -> Result<Vec<Track>, String> {
        let mut url = format!("{}/recommendations?track_id={}", API_BASE_URL, track_id);
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<Vec<Track>>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            Err(format!("Failed to fetch recommendations: {}", response.status()))
        }
    }

    pub async fn create_youtube_playlist(
        title: String,
        description: String,
        track_queries: Vec<String>,
    ) -> Result<CreatedPlaylist, String> {
        let request_body = YouTubePlaylistRequest {
            title,
            description,
            track_queries,
        };

        let response = Request::post(&format!("{}/youtube/playlist", API_BASE_URL))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<CreatedPlaylist>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Playlist creation failed: {}", error_text))
        }
    }

    pub async fn create_playlist_from_recommendations(
        title: String,
        description: String,
        track_id: String,
        limit: Option<u32>,
    ) -> Result<CreatedPlaylist, String> {
        let mut request_body = serde_json::json!({
            "title": title,
            "description": description,
            "track_id": track_id
        });

        if let Some(limit) = limit {
            request_body["limit"] = Value::from(limit);
        }

        let response = Request::post(&format!("{}/youtube/playlist/from-recommendations", API_BASE_URL))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<CreatedPlaylist>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Playlist creation failed: {}", error_text))
        }
    }

    pub async fn get_similar_tracks_with_youtube(track_id: String, limit: Option<u32>) -> Result<SimilarTracksResponse, String> {
        let mut url = format!("{}/similar-tracks?track_id={}", API_BASE_URL, track_id);
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }

        let response = Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.ok() {
            response
                .json::<SimilarTracksResponse>()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Failed to fetch similar tracks: {}", error_text))
        }
    }
}
