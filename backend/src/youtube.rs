use crate::models::{YouTubeVideo, CreatedPlaylist};
use crate::rate_limiter::{RateLimiter, RateLimitConfig};
use reqwest::Client;
use serde_json::{Value, json};
use anyhow::{Result, anyhow};
use std::sync::Arc;

pub struct YouTubeClient {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
}

impl YouTubeClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rate_limiter: Arc::new(RateLimiter::new(RateLimitConfig::youtube_config())),
        }
    }

    pub async fn search_video(&self, query: &str, api_key: &str) -> Result<Option<YouTubeVideo>> {
        let url = format!(
            "https://www.googleapis.com/youtube/v3/search?part=snippet&type=video&q={}&maxResults=1&key={}",
            urlencoding::encode(query),
            api_key
        );
        let client = &self.client;

        let response = self.rate_limiter.execute(|| async {
            client.get(&url).send().await.map_err(|e| anyhow!("Request failed: {}", e))
        }).await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("YouTube search failed: {}", response.status()));
        }

        let data: Value = response.json().await?;
        let items = data["items"].as_array().ok_or(anyhow!("Invalid YouTube response"))?;

        if items.is_empty() {
            return Ok(None);
        }

        let video = &items[0];
        let snippet = &video["snippet"];

        Ok(Some(YouTubeVideo {
            id: video["id"]["videoId"].as_str().ok_or(anyhow!("Missing video ID"))?.to_string(),
            title: snippet["title"].as_str().ok_or(anyhow!("Missing video title"))?.to_string(),
            channel_title: snippet["channelTitle"].as_str().ok_or(anyhow!("Missing channel title"))?.to_string(),
            duration: "Unknown".to_string(), // Would need additional API call to get duration
        }))
    }

    pub async fn create_playlist(&self, name: &str, description: Option<&str>, access_token: &str) -> Result<String> {
        let url = "https://www.googleapis.com/youtube/v3/playlists?part=snippet,status";
        
        let payload = json!({
            "snippet": {
                "title": name,
                "description": description.unwrap_or("Created by Spotify Neo4j Backend"),
                "defaultLanguage": "en"
            },
            "status": {
                "privacyStatus": "public"
            }
        });

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to create YouTube playlist: {}", response.status()));
        }

        let data: Value = response.json().await?;
        let playlist_id = data["id"].as_str().ok_or(anyhow!("Missing playlist ID"))?.to_string();

        Ok(playlist_id)
    }

    pub async fn add_video_to_playlist(&self, playlist_id: &str, video_id: &str, access_token: &str) -> Result<()> {
        let url = "https://www.googleapis.com/youtube/v3/playlistItems?part=snippet";
        
        let payload = json!({
            "snippet": {
                "playlistId": playlist_id,
                "resourceId": {
                    "kind": "youtube#video",
                    "videoId": video_id
                }
            }
        });

        let client = &self.client;
        let auth_header = format!("Bearer {}", access_token);

        let response = self.rate_limiter.execute(|| async {
            client
                .post(url)
                .header("Authorization", &auth_header)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| anyhow!("Request failed: {}", e))
        }).await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to add video to playlist: {}", response.status()));
        }

        Ok(())
    }

    pub async fn create_playlist_from_tracks(
        &self,
        name: &str,
        description: Option<&str>,
        track_names: &[String],
        access_token: &str,
        youtube_api_key: &str,
    ) -> Result<CreatedPlaylist> {
        // Create the playlist
        let playlist_id = self.create_playlist(name, description, access_token).await?;
        
        let mut tracks_added: i32 = 0;
        let mut tracks_not_found = Vec::new();

        // Process tracks in smaller batches to avoid overwhelming the API
        let batch_size = 10;
        for batch in track_names.chunks(batch_size) {
            for track_name in batch {
                match self.search_video(track_name, youtube_api_key).await {
                    Ok(Some(video)) => {
                        match self.add_video_to_playlist(&playlist_id, &video.id, access_token).await {
                            Ok(_) => {
                                tracks_added += 1;
                                println!("Added: {} - {}", track_name, video.title);
                            }
                            Err(e) => {
                                println!("Failed to add {} to playlist: {}", track_name, e);
                                tracks_not_found.push(track_name.clone());
                            }
                        }
                    }
                    Ok(None) => {
                        println!("No video found for: {}", track_name);
                        tracks_not_found.push(track_name.clone());
                    }
                    Err(e) => {
                        println!("Search failed for {}: {}", track_name, e);
                        tracks_not_found.push(track_name.clone());
                    }
                }
            }

            // Rate limiting is now handled by the RateLimiter, but add a small pause between batches
            if batch.len() == batch_size && tracks_added < track_names.len() as i32 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }

        Ok(CreatedPlaylist {
            id: playlist_id.clone(),
            name: name.to_string(),
            url: format!("https://www.youtube.com/playlist?list={}", playlist_id),
            tracks_added,
            tracks_not_found,
        })
    }

    pub fn format_search_query(track_name: &str, artist_names: &[String]) -> String {
        if artist_names.is_empty() {
            track_name.to_string()
        } else {
            format!("{} {}", artist_names.join(" "), track_name)
        }
    }
}
