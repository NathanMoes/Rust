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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spotify_neo4j_backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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
        .route("/api/similar-tracks", get(handlers::get_similar_tracks_with_youtube))
        .with_state(neo4j_client)
        // Serve static files from frontend/dist
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 Server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
