use crate::{
    models::*, 
    spotify::SpotifyClient, 
    neo4j_db::{self, Neo4jClient}, 
    youtube::YouTubeClient
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    Json as JsonBody,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn, error, debug, instrument};

// Helper function to extract playlist ID from Spotify URL
fn extract_playlist_id(url: &str) -> Option<String> {
    // Handle URLs like: https://open.spotify.com/playlist/441K4rF3u0qfg9m4X1WSQJ
    if let Some(captures) = url.split('/').last() {
        // Remove any query parameters
        let playlist_id = captures.split('?').next()?;
        if !playlist_id.is_empty() {
            return Some(playlist_id.to_string());
        }
    }
    None
}

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "Spotify Neo4j Backend",
        "version": "0.1.0"
    }))
}

#[instrument(skip(neo4j_client))]
pub async fn import_spotify_data(
    State(neo4j_client): State<Neo4jClient>,
    JsonBody(request): JsonBody<SpotifyImportRequest>,
) -> Result<Json<Value>, StatusCode> {
    let start_time = std::time::Instant::now();
    info!("Starting Spotify playlist import for URL: {}", request.playlist_url);
    
    let spotify_client = SpotifyClient::new();
    
    // Extract playlist ID from URL
    let playlist_id = match extract_playlist_id(&request.playlist_url) {
        Some(id) => {
            info!("Successfully extracted playlist ID: {}", id);
            id
        }
        None => {
            error!("Failed to extract playlist ID from URL: {}", request.playlist_url);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Try to get access token from environment variable first, then generate one if not found
    let access_token = match std::env::var("SPOTIFY_ACCESS_TOKEN") {
        Ok(token) => {
            debug!("Using Spotify access token from environment variable");
            token
        }
        Err(_) => {
            debug!("SPOTIFY_ACCESS_TOKEN not found in environment, generating new token");
            match spotify_client.get_access_token().await {
                Ok(token) => {
                    debug!("Successfully generated new Spotify access token");
                    token
                }
                Err(e) => {
                    error!("Failed to generate Spotify access token: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    };
    
    // Get tracks from Spotify playlist
    debug!("Fetching playlist tracks from Spotify API");
    let fetch_start = std::time::Instant::now();
    let tracks = match spotify_client
        .get_playlist_tracks(&playlist_id, &access_token)
        .await
    {
        Ok(tracks) => {
            let fetch_duration = fetch_start.elapsed();
            info!(
                "Successfully fetched {} tracks from playlist in {:.2}s", 
                tracks.len(), 
                fetch_duration.as_secs_f64()
            );
            tracks
        }
        Err(e) => {
            let fetch_duration = fetch_start.elapsed();
            error!(
                "Failed to fetch playlist tracks after {:.2}s: {}", 
                fetch_duration.as_secs_f64(),
                e
            );
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let mut imported_tracks = 0;
    let mut imported_artists = 0;
    let mut processed_artists = std::collections::HashSet::new();

    info!("Starting database storage for {} tracks", tracks.len());
    let storage_start = std::time::Instant::now();

    // Store tracks and artists in Neo4j
    for (track_index, track) in tracks.iter().enumerate() {
        debug!("Processing track {}/{}: {}", track_index + 1, tracks.len(), track.name);
        
        // Store track
        let track_store_start = std::time::Instant::now();
        match neo4j_db::store_track(&neo4j_client, track).await {
            Ok(_) => {
                imported_tracks += 1;
                debug!(
                    "Stored track '{}' in {:.3}s", 
                    track.name, 
                    track_store_start.elapsed().as_secs_f64()
                );
            }
            Err(e) => {
                error!("Failed to store track '{}': {}", track.name, e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        // Store artists (avoid duplicates)
        for artist_id in &track.artist_ids {
            if !processed_artists.contains(artist_id) {
                debug!("Fetching artist details for ID: {}", artist_id);
                let artist_fetch_start = std::time::Instant::now();
                
                match spotify_client.get_artist(artist_id, &access_token).await {
                    Ok(artist) => {
                        let fetch_duration = artist_fetch_start.elapsed();
                        debug!(
                            "Fetched artist '{}' in {:.3}s", 
                            artist.name, 
                            fetch_duration.as_secs_f64()
                        );
                        
                        let artist_store_start = std::time::Instant::now();
                        match neo4j_db::store_artist(&neo4j_client, &artist).await {
                            Ok(_) => {
                                imported_artists += 1;
                                processed_artists.insert(artist_id.clone());
                                debug!(
                                    "Stored artist '{}' in {:.3}s", 
                                    artist.name, 
                                    artist_store_start.elapsed().as_secs_f64()
                                );
                            }
                            Err(e) => {
                                error!("Failed to store artist '{}': {}", artist.name, e);
                                return Err(StatusCode::INTERNAL_SERVER_ERROR);
                            }
                        }
                    }
                    Err(e) => {
                        let fetch_duration = artist_fetch_start.elapsed();
                        warn!(
                            "Failed to fetch artist {} after {:.3}s: {}", 
                            artist_id, 
                            fetch_duration.as_secs_f64(),
                            e
                        );
                    }
                }
            }
        }
    }

    let storage_duration = storage_start.elapsed();
    let total_duration = start_time.elapsed();
    
    info!(
        "Spotify import completed successfully in {:.2}s (storage: {:.2}s). Imported {} tracks and {} artists from playlist {}",
        total_duration.as_secs_f64(),
        storage_duration.as_secs_f64(),
        imported_tracks,
        imported_artists,
        playlist_id
    );

    Ok(Json(json!({
        "message": "Spotify data imported successfully",
        "imported_tracks": imported_tracks,
        "imported_artists": imported_artists,
        "playlist_id": playlist_id,
        "duration_seconds": total_duration.as_secs_f64()
    })))
}

pub async fn get_artists(
    State(neo4j_client): State<Neo4jClient>,
) -> Result<Json<Vec<Artist>>, StatusCode> {
    let artists = neo4j_db::get_all_artists(&neo4j_client)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(artists))
}

pub async fn get_tracks(
    State(neo4j_client): State<Neo4jClient>,
) -> Result<Json<Vec<Track>>, StatusCode> {
    let tracks = neo4j_db::get_all_tracks(&neo4j_client)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tracks))
}

pub async fn get_recommendations(
    State(neo4j_client): State<Neo4jClient>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Track>>, StatusCode> {
    let seed_tracks: Vec<String> = params
        .get("seed_tracks")
        .map(|s| s.split(',').map(|id| id.trim().to_string()).collect())
        .unwrap_or_default();

    if seed_tracks.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);

    let recommendations = neo4j_db::get_similar_tracks(&neo4j_client, &seed_tracks, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(recommendations))
}

pub async fn create_youtube_playlist(
    JsonBody(request): JsonBody<YouTubePlaylistRequest>,
) -> Result<Json<CreatedPlaylist>, StatusCode> {
    let youtube_client = YouTubeClient::new();
    let youtube_api_key = std::env::var("YOUTUBE_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let playlist = youtube_client
        .create_playlist_from_tracks(
            &request.name,
            request.description.as_deref(),
            &request.track_names,
            &request.access_token,
            &youtube_api_key,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(playlist))
}

// Additional handler for creating YouTube playlist from Neo4j recommendations
pub async fn create_youtube_playlist_from_recommendations(
    State(neo4j_client): State<Neo4jClient>,
    JsonBody(request): JsonBody<Value>,
) -> Result<Json<CreatedPlaylist>, StatusCode> {
    // Extract parameters from request
    let seed_tracks: Vec<String> = request["seed_tracks"]
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();

    let playlist_name = request["playlist_name"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let youtube_access_token = request["youtube_access_token"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let limit = request["limit"]
        .as_i64()
        .unwrap_or(20) as i32;

    // Get recommendations from Neo4j
    let recommendations = neo4j_db::get_similar_tracks(&neo4j_client, &seed_tracks, limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Format track names for YouTube search
    let track_names: Vec<String> = recommendations
        .iter()
        .map(|track| YouTubeClient::format_search_query(&track.name, &track.artist_names))
        .collect();

    // Create YouTube playlist
    let youtube_client = YouTubeClient::new();
    let youtube_api_key = std::env::var("YOUTUBE_API_KEY")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let playlist = youtube_client
        .create_playlist_from_tracks(
            playlist_name,
            Some("Generated from Spotify recommendations via Neo4j"),
            &track_names,
            youtube_access_token,
            &youtube_api_key,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(playlist))
}

pub async fn get_similar_tracks_with_youtube(
    State(neo4j_client): State<Neo4jClient>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SimilarTracksResponse>, StatusCode> {
    let track_id = params
        .get("track_id")
        .ok_or(StatusCode::BAD_REQUEST)?;

    let limit = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // Get the original track
    let original_track = neo4j_db::get_track_by_id(&neo4j_client, track_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get similar tracks
    let similar_tracks = neo4j_db::get_similar_tracks(&neo4j_client, &[track_id.clone()], limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get YouTube API key from environment
    let youtube_api_key = std::env::var("YOUTUBE_API_KEY").ok();
    let youtube_client = YouTubeClient::new();

    let mut tracks_with_youtube = Vec::new();

    // Search for each similar track on YouTube
    for track in similar_tracks {
        let youtube_video = if let Some(ref api_key) = youtube_api_key {
            let search_query = YouTubeClient::format_search_query(&track.name, &track.artist_names);
            youtube_client.search_video(&search_query, api_key)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        tracks_with_youtube.push(TrackWithYouTube {
            track,
            youtube_video,
        });
    }

    Ok(Json(SimilarTracksResponse {
        original_track,
        similar_tracks: tracks_with_youtube,
    }))
}
