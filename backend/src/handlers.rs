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

pub async fn import_spotify_data(
    State(neo4j_client): State<Neo4jClient>,
    JsonBody(request): JsonBody<SpotifyImportRequest>,
) -> Result<Json<Value>, StatusCode> {
    let spotify_client = SpotifyClient::new();
    
    // Extract playlist ID from URL
    let playlist_id = extract_playlist_id(&request.playlist_url)
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    // For now, we'll need to handle authentication differently
    // This is a placeholder - in a real app, you'd get this from the user
    let access_token = std::env::var("SPOTIFY_ACCESS_TOKEN")
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Get tracks from Spotify playlist
    let tracks = spotify_client
        .get_playlist_tracks(&playlist_id, &access_token)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut imported_tracks = 0;
    let mut imported_artists = 0;
    let mut processed_artists = std::collections::HashSet::new();

    // Store tracks and artists in Neo4j
    for track in &tracks {
        // Store track
        neo4j_db::store_track(&neo4j_client, track)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        imported_tracks += 1;

        // Store artists (avoid duplicates)
        for artist_id in &track.artist_ids {
            if !processed_artists.contains(artist_id) {
                match spotify_client.get_artist(artist_id, &access_token).await {
                    Ok(artist) => {
                        neo4j_db::store_artist(&neo4j_client, &artist)
                            .await
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        
                        imported_artists += 1;
                        processed_artists.insert(artist_id.clone());
                    }
                    Err(e) => {
                        println!("Warning: Failed to fetch artist {}: {}", artist_id, e);
                    }
                }
            }
        }
    }

    Ok(Json(json!({
        "message": "Spotify data imported successfully",
        "imported_tracks": imported_tracks,
        "imported_artists": imported_artists,
        "playlist_id": playlist_id
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
