#!/bin/bash

echo "🎵 Spotify Neo4j Backend Setup"
echo "================================"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

echo "🐳 Starting Neo4j database..."
docker-compose up -d neo4j

echo "⏳ Waiting for Neo4j to be ready (this may take 30-60 seconds)..."
timeout=60
counter=0
while ! docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "RETURN 1;" > /dev/null 2>&1; do
    sleep 2
    counter=$((counter + 2))
    if [ $counter -ge $timeout ]; then
        echo "❌ Neo4j failed to start within $timeout seconds"
        exit 1
    fi
    echo "   Still waiting... ($counter/$timeout seconds)"
done

echo "✅ Neo4j is ready!"
echo "🌐 Neo4j Browser: http://localhost:7474"
echo "   Username: neo4j"
echo "   Password: password123"
echo ""

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📄 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env file with your API credentials:"
    echo "   - YOUTUBE_API_KEY: Get from Google Cloud Console"
    echo "   - Add Spotify credentials if you plan to use Spotify integration"
    echo ""
fi

echo "🚀 Starting Rust backend server..."
echo "   Backend will be available at: http://localhost:3000"
echo "   API endpoints:"
echo "   - GET  /                                           - Health check"
echo "   - POST /spotify/import                             - Import Spotify data"
echo "   - GET  /spotify/artists                            - Get all artists"
echo "   - GET  /spotify/tracks                             - Get all tracks"
echo "   - GET  /recommendations?track_id=TRACK_ID          - Get recommendations"
echo "   - POST /youtube/playlist                           - Create YouTube playlist"
echo "   - POST /youtube/playlist/from-recommendations      - Create playlist from recommendations"
echo ""

cargo run
