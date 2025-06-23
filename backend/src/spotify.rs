use crate::models::{Artist, Track};
use crate::rate_limiter::{RateLimiter, RateLimitConfig};
use reqwest::Client;
use serde_json::Value;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug, instrument};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

pub struct SpotifyClient {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rate_limiter: Arc::new(RateLimiter::new(RateLimitConfig::spotify_config())),
        }
    }

    /// Get access token using Client Credentials flow
    #[instrument(skip(self))]
    pub async fn get_access_token(&self) -> Result<String> {
        let client_id = std::env::var("SPOTIFY_CLIENT_ID")
            .map_err(|_| anyhow!("SPOTIFY_CLIENT_ID environment variable not found"))?;
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
            .map_err(|_| anyhow!("SPOTIFY_CLIENT_SECRET environment variable not found"))?;

        debug!("Requesting new Spotify access token");

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ];

        let response = self.client
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Failed to get Spotify access token: {}", error_text);
            return Err(anyhow!("Failed to get Spotify access token: {}", error_text));
        }

        let token_response: TokenResponse = response.json().await?;
        info!("Successfully obtained Spotify access token (expires in {} seconds)", token_response.expires_in);
        
        Ok(token_response.access_token)
    }

    #[instrument(skip(self, access_token), fields(playlist_id = %playlist_id))]
    pub async fn get_playlist_tracks(&self, playlist_id: &str, access_token: &str) -> Result<Vec<Track>> {
        debug!("Starting playlist tracks fetch");
        let mut tracks = Vec::new();
        let mut offset = 0;
        let limit = 50;
        let mut page_count = 0;

        loop {
            page_count += 1;
            let url = format!(
                "https://api.spotify.com/v1/playlists/{}/tracks?offset={}&limit={}",
                playlist_id, offset, limit
            );

            debug!("Fetching playlist page {} (offset: {}, limit: {})", page_count, offset, limit);
            let request_start = std::time::Instant::now();
            let client = &self.client;
            let auth_header = format!("Bearer {}", access_token);

            let response = self.rate_limiter.execute(|| async {
                client
                    .get(&url)
                    .header("Authorization", &auth_header)
                    .send()
                    .await
                    .map_err(|e| anyhow!("Network error: {}", e))
            }).await?;

            let request_duration = request_start.elapsed();
            debug!(
                "Spotify API request completed in {:.3}s with status: {}", 
                request_duration.as_secs_f64(), 
                response.status()
            );

            if !response.status().is_success() {
                let status = response.status();
                error!("Spotify API returned error status: {}", status);
                let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error body".to_string());
                error!("Error response body: {}", error_body);
                return Err(anyhow!("Failed to fetch playlist tracks: {} - {}", status, error_body));
            }

            let parse_start = std::time::Instant::now();
            let data: Value = match response.json().await {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to parse JSON response: {}", e);
                    return Err(anyhow!("Invalid JSON response: {}", e));
                }
            };
            
            debug!("Parsed JSON response in {:.3}s", parse_start.elapsed().as_secs_f64());
            
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);
            let items_count = items.len();

            debug!("Processing {} tracks from page {}", items_count, page_count);

            if items.is_empty() {
                info!("No more tracks found, ending pagination");
                break;
            }

            let mut page_tracks = 0;
            let mut page_errors = 0;
            for item in items {
                if let Some(track_data) = item["track"].as_object() {
                    match self.parse_track(track_data, access_token).await {
                        Ok(track) => {
                            tracks.push(track);
                            page_tracks += 1;
                        }
                        Err(e) => {
                            page_errors += 1;
                            warn!("Failed to parse track: {}", e);
                        }
                    }
                }
            }

            debug!("Page {} completed: {} tracks parsed, {} errors", page_count, page_tracks, page_errors);
            offset += limit;
        }

        info!("Playlist tracks fetch completed: {} total tracks from {} pages", tracks.len(), page_count);
        Ok(tracks)
    }

    async fn parse_track(&self, track_data: &serde_json::Map<String, Value>, access_token: &str) -> Result<Track> {
        let id = track_data["id"].as_str().ok_or(anyhow!("Missing track id"))?.to_string();
        let name = track_data["name"].as_str().ok_or(anyhow!("Missing track name"))?.to_string();
        
        let empty_vec = vec![];
        let artists = track_data["artists"].as_array().unwrap_or(&empty_vec);
        let mut artist_ids = Vec::new();
        let mut artist_names = Vec::new();
        
        for artist in artists {
            if let Some(artist_id) = artist["id"].as_str() {
                artist_ids.push(artist_id.to_string());
            }
            if let Some(artist_name) = artist["name"].as_str() {
                artist_names.push(artist_name.to_string());
            }
        }

        let album = &track_data["album"];
        let album_id = album["id"].as_str().unwrap_or("").to_string();
        let album_name = album["name"].as_str().unwrap_or("").to_string();
        
        // Get audio features
        let audio_features = self.get_audio_features(&id, access_token).await?;

        Ok(Track {
            id,
            name,
            artist_ids,
            artist_names,
            album_id,
            album_name,
            duration_ms: track_data["duration_ms"].as_i64().unwrap_or(0) as i32,
            popularity: track_data["popularity"].as_i64().unwrap_or(0) as i32,
            explicit: track_data["explicit"].as_bool().unwrap_or(false),
            danceability: audio_features["danceability"].as_f64().unwrap_or(0.0),
            energy: audio_features["energy"].as_f64().unwrap_or(0.0),
            key: audio_features["key"].as_i64().unwrap_or(0) as i32,
            loudness: audio_features["loudness"].as_f64().unwrap_or(0.0),
            mode: audio_features["mode"].as_i64().unwrap_or(0) as i32,
            speechiness: audio_features["speechiness"].as_f64().unwrap_or(0.0),
            acousticness: audio_features["acousticness"].as_f64().unwrap_or(0.0),
            instrumentalness: audio_features["instrumentalness"].as_f64().unwrap_or(0.0),
            liveness: audio_features["liveness"].as_f64().unwrap_or(0.0),
            valence: audio_features["valence"].as_f64().unwrap_or(0.0),
            tempo: audio_features["tempo"].as_f64().unwrap_or(0.0),
            time_signature: audio_features["time_signature"].as_i64().unwrap_or(4) as i32,
            preview_url: track_data["preview_url"].as_str().map(|s| s.to_string()),
        })
    }

    async fn get_audio_features(&self, track_id: &str, access_token: &str) -> Result<Value> {
        let url = format!("https://api.spotify.com/v1/audio-features/{}", track_id);
        let client = &self.client;
        let auth_header = format!("Bearer {}", access_token);
        
        self.rate_limiter.execute(|| async {
            let response = client
                .get(&url)
                .header("Authorization", &auth_header)
                .send()
                .await
                .map_err(|e| anyhow!("Request failed: {}", e))?;

            if response.status().is_success() {
                response.json().await.map_err(|e| anyhow!("JSON parse failed: {}", e))
            } else {
                // Return empty object if audio features not available
                Ok(serde_json::json!({}))
            }
        }).await
    }

    #[instrument(skip(self, access_token), fields(artist_id = %artist_id))]
    pub async fn get_artist(&self, artist_id: &str, access_token: &str) -> Result<Artist> {
        debug!("Fetching artist details");
        let url = format!("https://api.spotify.com/v1/artists/{}", artist_id);
        let client = &self.client;
        let auth_header = format!("Bearer {}", access_token);
        
        let request_start = std::time::Instant::now();
        let response = self.rate_limiter.execute(|| async {
            client
                .get(&url)
                .header("Authorization", &auth_header)
                .send()
                .await
                .map_err(|e| anyhow!("Network error: {}", e))
        }).await?;

        let request_duration = request_start.elapsed();
        debug!(
            "Artist API request completed in {:.3}s with status: {}", 
            request_duration.as_secs_f64(), 
            response.status()
        );

        if !response.status().is_success() {
            let status = response.status();
            error!("Spotify Artist API returned error status: {}", status);
            let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error body".to_string());
            error!("Error response body: {}", error_body);
            return Err(anyhow!("Failed to fetch artist: {} - {}", status, error_body));
        }

        let parse_start = std::time::Instant::now();
        let data: Value = match response.json().await {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse artist JSON response: {}", e);
                return Err(anyhow!("Invalid JSON response: {}", e));
            }
        };
        
        debug!("Parsed artist JSON response in {:.3}s", parse_start.elapsed().as_secs_f64());
        
        let genres = data["genres"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let image_url = data["images"].as_array()
            .and_then(|images| images.first())
            .and_then(|img| img["url"].as_str())
            .map(|s| s.to_string());

        let artist_name = data["name"].as_str().ok_or(anyhow!("Missing artist name"))?.to_string();
        debug!("Successfully parsed artist: {}", artist_name);

        Ok(Artist {
            id: data["id"].as_str().ok_or(anyhow!("Missing artist id"))?.to_string(),
            name: artist_name,
            genres,
            popularity: data["popularity"].as_i64().unwrap_or(0) as i32,
            followers: data["followers"]["total"].as_i64().unwrap_or(0) as i32,
            image_url,
        })
    }

    pub async fn get_recommendations(&self, 
        seed_tracks: &[String], 
        target_valence: Option<f64>,
        target_energy: Option<f64>,
        target_danceability: Option<f64>,
        limit: i32,
        access_token: &str
    ) -> Result<Vec<Track>> {
        let mut url = format!(
            "https://api.spotify.com/v1/recommendations?seed_tracks={}&limit={}",
            seed_tracks.join(","),
            limit
        );

        if let Some(valence) = target_valence {
            url.push_str(&format!("&target_valence={}", valence));
        }
        if let Some(energy) = target_energy {
            url.push_str(&format!("&target_energy={}", energy));
        }
        if let Some(danceability) = target_danceability {
            url.push_str(&format!("&target_danceability={}", danceability));
        }

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get recommendations: {}", response.status()));
        }

        let data: Value = response.json().await?;
        let tracks = data["tracks"].as_array().ok_or(anyhow!("Invalid response format"))?;
        
        let mut result_tracks = Vec::new();
        for track_value in tracks {
            if let Ok(track) = self.parse_track(track_value.as_object().unwrap(), access_token).await {
                result_tracks.push(track);
            }
        }

        Ok(result_tracks)
    }
}
