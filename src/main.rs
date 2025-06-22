mod spotify;
mod neo4j_db;
mod youtube;
mod models;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Initialize Neo4j connection
    let neo4j_client = neo4j_db::init_neo4j().await?;
    
    // Create routes
    let app = Router::new()
        .route("/", get(handlers::health_check))
        .route("/spotify/import", post(handlers::import_spotify_data))
        .route("/spotify/artists", get(handlers::get_artists))
        .route("/spotify/tracks", get(handlers::get_tracks))
        .route("/youtube/playlist", post(handlers::create_youtube_playlist))
        .route("/youtube/playlist/from-recommendations", post(handlers::create_youtube_playlist_from_recommendations))
        .route("/recommendations", get(handlers::get_recommendations))
        .layer(CorsLayer::permissive())
        .with_state(neo4j_client);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
