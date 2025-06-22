use crate::models::{Artist, Track};
use reqwest::Client;
use serde_json::Value;
use anyhow::{Result, anyhow};

pub struct SpotifyClient {
    client: Client,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str, access_token: &str) -> Result<Vec<Track>> {
        let mut tracks = Vec::new();
        let mut offset = 0;
        let limit = 50;

        loop {
            let url = format!(
                "https://api.spotify.com/v1/playlists/{}/tracks?offset={}&limit={}",
                playlist_id, offset, limit
            );

            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow!("Failed to fetch playlist tracks: {}", response.status()));
            }

            let data: Value = response.json().await?;
            let empty_vec = vec![];
            let items = data["items"].as_array().unwrap_or(&empty_vec);

            if items.is_empty() {
                break;
            }

            for item in items {
                if let Some(track_data) = item["track"].as_object() {
                    if let Ok(track) = self.parse_track(track_data, access_token).await {
                        tracks.push(track);
                    }
                }
            }

            offset += limit;
        }

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
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            // Return empty object if audio features not available
            Ok(serde_json::json!({}))
        }
    }

    pub async fn get_artist(&self, artist_id: &str, access_token: &str) -> Result<Artist> {
        let url = format!("https://api.spotify.com/v1/artists/{}", artist_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch artist: {}", response.status()));
        }

        let data: Value = response.json().await?;
        
        let genres = data["genres"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let image_url = data["images"].as_array()
            .and_then(|images| images.first())
            .and_then(|img| img["url"].as_str())
            .map(|s| s.to_string());

        Ok(Artist {
            id: data["id"].as_str().ok_or(anyhow!("Missing artist id"))?.to_string(),
            name: data["name"].as_str().ok_or(anyhow!("Missing artist name"))?.to_string(),
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
