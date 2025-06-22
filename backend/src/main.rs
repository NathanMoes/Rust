mod spotify;
mod neo4j_db;
mod youtube;
mod models;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Initialize Neo4j connection
    let neo4j_client = neo4j_db::init_neo4j().await?;
    
    // Create routes
    let app = Router::new()
        // API routes
        .route("/api/health", get(handlers::health_check))
        .route("/api/spotify/import", post(handlers::import_spotify_data))
        .route("/api/spotify/artists", get(handlers::get_artists))
        .route("/api/spotify/tracks", get(handlers::get_tracks))
        .route("/api/youtube/playlist", post(handlers::create_youtube_playlist))
        .route("/api/youtube/playlist/from-recommendations", post(handlers::create_youtube_playlist_from_recommendations))
        .route("/api/recommendations", get(handlers::get_recommendations))
        .with_state(neo4j_client)
        // Serve static files from frontend/dist
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
