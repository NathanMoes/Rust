# Spotify Neo4j Full-Stack Application

A complete full-stack Rust application that integrates Spotify music data with Neo4j graph database to create intelligent YouTube playlists. Features a WebAssembly frontend built with Yew and a high-performance Axum backend.

## 🌟 Features

- 🎵 **Spotify Integration**: Import playlist data, tracks, and audio features
- 🔗 **Neo4j Graph Database**: Store music relationships and audio characteristics  
- 🎯 **Smart Recommendations**: Find similar tracks based on audio features
- 📺 **YouTube Playlist Creation**: Automatically create YouTube playlists
- 🦀 **Full-Stack Rust**: WebAssembly frontend + Axum backend
- ⚡ **Modern UI**: Responsive web interface built with Yew and Tailwind CSS
- 🔄 **Real-time Updates**: Interactive frontend with real-time API communication
- 🚀 **Fast API**: Built with Axum for high-performance HTTP handling

## Project Structure

```
spotify-neo4j-app/
├── backend/                    # Rust Axum API server
│   ├── src/
│   │   ├── main.rs
│   │   ├── handlers.rs
│   │   ├── models.rs
│   │   ├── neo4j_db.rs
│   │   ├── spotify.rs
│   │   └── youtube.rs
│   ├── Cargo.toml
│   └── target/
├── frontend/                   # Yew WebAssembly frontend
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── types.rs
│   │   ├── components/
│   │   ├── pages/
│   │   └── services/
│   ├── Cargo.toml
│   ├── index.html
│   └── dist/
├── docker-compose.yml          # Neo4j database
├── dev.sh                     # Development setup script
├── .env.example               # Environment variables template
└── README.md
```

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Spotify API   │────▶│   Rust Backend  │────▶│   Neo4j Graph   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                │
                                ▼
                        ┌─────────────────┐
                        │  YouTube API    │
                        └─────────────────┘
                                │
                                ▼
                        ┌─────────────────┐
                        │ WebAssembly UI  │
                        │   (Yew + Rust)  │
                        └─────────────────┘
```

## 🚀 Quick Start

### Method 1: One-Command Setup (Recommended)
```bash
# Start everything in development mode
./dev.sh

# Or build for production
./dev.sh build
```

### Method 2: Manual Setup
```bash
# 1. Install tools
./dev.sh install

# 2. Start Neo4j database
docker compose up -d neo4j

# 3. Set up environment
cp .env.example .env
# Edit .env with your API keys

# 4. Start backend
cd backend && cargo run

# 5. Start frontend (in another terminal)
cd frontend && trunk serve
```

## 🌐 Access Points

- **Frontend**: http://localhost:8080 (Development) or http://localhost:3000 (Production)
- **Backend API**: http://localhost:3000/api/
- **Neo4j Browser**: http://localhost:7474 (`neo4j` / `password123`)

## Prerequisites

1. **Neo4j Database**: Install and run Neo4j
2. **Spotify Developer Account**: For API access
3. **YouTube API Key**: For playlist creation
4. **Rust**: Latest stable version

## Installation

1. **Clone and setup**:
```bash
git clone <your-repo>
cd spotify-neo4j-backend
cp .env.example .env
```

2. **Configure environment variables** in `.env`:
```env
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=your_password
YOUTUBE_API_KEY=your_youtube_api_key
```

3. **Install and run**:
```bash
cargo build
cargo run
```

## 📁 Project Structure

```
spotify-neo4j-backend/
├── src/                    # Backend source code
│   ├── main.rs            # Axum server setup
│   ├── handlers.rs        # HTTP request handlers
│   ├── models.rs          # Data structures
│   ├── spotify.rs         # Spotify API client
│   ├── neo4j_db.rs       # Neo4j operations
│   └── youtube.rs         # YouTube API client
├── frontend/              # WebAssembly frontend
│   ├── src/
│   │   ├── main.rs       # Yew app entry point
│   │   ├── components/   # UI components
│   │   ├── pages/        # Page components
│   │   ├── services/     # API services
│   │   └── types.rs      # Type definitions
│   ├── index.html        # HTML template
│   ├── Cargo.toml        # Frontend dependencies
│   └── Trunk.toml        # Build configuration
├── Cargo.toml             # Backend dependencies
├── docker-compose.yml     # Neo4j setup
├── dev.sh                 # Development script
└── .env.example          # Environment template
```

## 🎯 Frontend Features

- **🎨 Modern UI**: Built with Yew (Rust WebAssembly framework) and Tailwind CSS
- **📱 Responsive Design**: Works on desktop and mobile devices
- **⚡ Fast Performance**: WebAssembly provides near-native performance
- **🔄 Real-time Updates**: Seamless API integration with the Rust backend
- **🎵 Music Visualization**: Interactive displays of audio features and recommendations
- **📊 Data Visualization**: Visual representation of music relationships and graph data

## 🔧 Backend Features

- **⚡ High Performance**: Built with Axum for maximum throughput
- **🔐 Type Safety**: Full Rust type system benefits
- **🗄️ GraphQL-style Queries**: Neo4j Cypher integration
- **🔄 Async/Await**: Non-blocking I/O operations
- **📡 RESTful API**: Clean HTTP endpoint design
- **🔗 CORS Support**: Cross-origin resource sharing enabled

## API Endpoints

### Health Check
```http
GET /
```

### Import Spotify Data
```http
POST /spotify/import
Content-Type: application/json

{
  "playlist_id": "spotify_playlist_id",
  "access_token": "spotify_access_token"
}
```

### Get Artists
```http
GET /spotify/artists
```

### Get Tracks
```http
GET /spotify/tracks
```

### Get Recommendations
```http
GET /recommendations?seed_tracks=track_id1,track_id2&limit=20
```

### Create YouTube Playlist
```http
POST /youtube/playlist
Content-Type: application/json

{
  "name": "My Playlist",
  "description": "Generated playlist",
  "track_names": ["Artist - Song", "Artist2 - Song2"],
  "access_token": "youtube_access_token"
}
```

### Create YouTube Playlist from Recommendations
```http
POST /youtube/playlist/from-recommendations
Content-Type: application/json

{
  "seed_tracks": ["track_id1", "track_id2"],
  "playlist_name": "AI Generated Playlist",
  "youtube_access_token": "youtube_access_token",
  "limit": 20
}
```

## Usage Workflow

1. **Setup Authentication**:
   - Get Spotify access token via OAuth2
   - Get YouTube access token via OAuth2
   - Obtain YouTube API key from Google Cloud Console

2. **Import Spotify Data**:
   ```bash
   curl -X POST http://localhost:3000/spotify/import \
     -H "Content-Type: application/json" \
     -d '{
       "playlist_id": "your_spotify_playlist_id",
       "access_token": "your_spotify_token"
     }'
   ```

3. **Get Recommendations**:
   ```bash
   curl "http://localhost:3000/recommendations?seed_tracks=track_id1,track_id2&limit=10"
   ```

4. **Create YouTube Playlist**:
   ```bash
   curl -X POST http://localhost:3000/youtube/playlist/from-recommendations \
     -H "Content-Type: application/json" \
     -d '{
       "seed_tracks": ["track_id1", "track_id2"],
       "playlist_name": "My AI Playlist",
       "youtube_access_token": "your_youtube_token",
       "limit": 20
     }'
   ```

## Neo4j Graph Schema

The application creates the following graph structure:

```cypher
(:Artist)-[:PERFORMED]->(:Track)
(:Album)-[:CONTAINS]->(:Track)
```

### Node Properties

**Artist**:
- `id`: Spotify artist ID
- `name`: Artist name
- `genres`: Array of genre strings
- `popularity`: Popularity score (0-100)
- `followers`: Number of followers
- `image_url`: Artist image URL

**Track**:
- `id`: Spotify track ID
- `name`: Track name
- `duration_ms`: Duration in milliseconds
- `popularity`: Popularity score (0-100)
- `explicit`: Boolean for explicit content
- Audio features: `danceability`, `energy`, `valence`, `tempo`, etc.

**Album**:
- `id`: Spotify album ID
- `name`: Album name

## Audio Feature Analysis

The system analyzes these Spotify audio features for recommendations:

- **Danceability** (0.0-1.0): How suitable for dancing
- **Energy** (0.0-1.0): Intensity and power
- **Valence** (0.0-1.0): Musical positivity
- **Tempo** (BPM): Speed of the track
- **Acousticness** (0.0-1.0): Acoustic vs electronic
- **Instrumentalness** (0.0-1.0): Vocal vs instrumental content

## Development

### Running in Development
```bash
cargo watch -x run
```

### Testing
```bash
cargo test
```

### Database Queries
Access your Neo4j browser at `http://localhost:7474` and run:

```cypher
// Find all artists
MATCH (a:Artist) RETURN a LIMIT 10

// Find tracks by energy level
MATCH (t:Track) WHERE t.energy > 0.8 RETURN t LIMIT 10

// Find similar tracks to a seed track
MATCH (seed:Track {id: "your_track_id"})
MATCH (similar:Track)
WHERE similar.id <> seed.id
WITH similar, seed,
     abs(similar.valence - seed.valence) as valence_diff,
     abs(similar.energy - seed.energy) as energy_diff
ORDER BY valence_diff + energy_diff ASC
LIMIT 10
RETURN similar
```

## Dependencies

- **neo4rs**: Neo4j driver for Rust
- **axum**: Modern web framework
- **reqwest**: HTTP client for API calls
- **serde**: Serialization framework
- **tokio**: Async runtime

## Error Handling

The application includes comprehensive error handling for:
- Spotify API rate limiting and errors
- Neo4j connection issues
- YouTube API quota limits
- Invalid authentication tokens

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Troubleshooting

### Common Issues

1. **Neo4j Connection Failed**:
   - Ensure Neo4j is running
   - Check credentials in `.env`
   - Verify network connectivity

2. **Spotify API Errors**:
   - Ensure access token is valid and not expired
   - Check playlist permissions
   - Verify playlist ID format

3. **YouTube API Quota Exceeded**:
   - YouTube API has daily quota limits
   - Consider implementing rate limiting
   - Use quota-efficient endpoints

4. **Track Not Found on YouTube**:
   - Some tracks may not be available
   - The service handles these gracefully
   - Check the `tracks_not_found` field in responses
