#!/bin/bash

echo "🎵 Spotify Neo4j Full-Stack Development Setup"
echo "=============================================="

MODE="${1:-dev}"

# Add Cargo bin to PATH if not already there
export PATH="$HOME/.cargo/bin:$PATH"

# Configure Rust toolchain and install Trunk
rustup default stable || true # ignore failure if already configured
cargo install trunk

install_tools() {
    echo "🔧 Installing required tools..."
    
    # Add Cargo bin to PATH if not already there
    export PATH="$HOME/.cargo/bin:$PATH"
    
    # Install Trunk for WebAssembly frontend builds
    if ! command -v trunk &> /dev/null; then
        echo "Installing Trunk..."
        cargo install trunk
    fi
    
    # Add wasm32 target
    rustup target add wasm32-unknown-unknown
    
    echo "✅ Tools installed successfully!"
}

cleanup_docker() {
    echo "🧹 Cleaning up Docker resources..."
    
    # Stop and remove Neo4j container
    if docker ps -a | grep -q spotify-neo4j; then
        echo "Stopping and removing Neo4j container..."
        docker stop spotify-neo4j 2>/dev/null || true
        docker rm spotify-neo4j 2>/dev/null || true
    fi
    
    # Clean up unused Docker resources
    echo "Cleaning up unused Docker resources..."
    docker system prune -f
    
    # Remove volumes if requested
    read -p "🗑️  Remove Neo4j data volumes? This will delete all stored data! (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Removing Neo4j volumes..."
        docker volume rm spotify-neo4j_neo4j_data spotify-neo4j_neo4j_logs spotify-neo4j_neo4j_import spotify-neo4j_neo4j_plugins 2>/dev/null || true
        docker compose -f docker-compose.yml down -v
    fi
    
    echo "✅ Docker cleanup complete!"
}

build_frontend() {
    echo "🔨 Building frontend..."
    cd frontend
    trunk build --release
    cd ..
    echo "✅ Frontend built!"
}

build_backend() {
    echo "🔨 Building backend..."
    cd backend
    cargo build --release
    cd ..
    echo "✅ Backend built!"
}

start_services() {
    echo "🐳 Starting services..."
    
    # Clean up any failed containers first
    if docker ps -a | grep -q spotify-neo4j; then
        echo "🧹 Cleaning up existing Neo4j container..."
        docker stop spotify-neo4j 2>/dev/null || true
        docker rm spotify-neo4j 2>/dev/null || true
    fi
    
    # Start Neo4j fresh
    echo "🚀 Starting Neo4j database..."
    if ! docker compose -f docker-compose.yml up -d neo4j; then
        echo "❌ Failed to start Neo4j container"
        echo "💡 Checking Docker system status..."
        docker system df
        echo ""
        echo "🔧 Attempting to clean up and retry..."
        docker system prune -f --volumes
        docker compose -f docker-compose.yml up -d neo4j
    fi
    
    # Wait for Neo4j to be ready with improved error handling
    echo "⏳ Waiting for Neo4j to be ready..."
    timeout=180
    counter=0
    health_check_interval=5
    
    while [ $counter -lt $timeout ]; do
        # Check if container is running
        if ! docker ps | grep -q spotify-neo4j; then
            echo "❌ Neo4j container stopped unexpectedly"
            echo "💡 Container logs:"
            docker logs spotify-neo4j --tail=10
            exit 1
        fi
        
        # Check if Neo4j is responding
        if docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "RETURN 1;" > /dev/null 2>&1; then
            echo "✅ Neo4j is ready!"
            return 0
        fi
        
        sleep $health_check_interval
        counter=$((counter + health_check_interval))
        echo "   Still waiting... ($counter/$timeout seconds)"
        
        # Show progress every 30 seconds
        if [ $((counter % 30)) -eq 0 ]; then
            echo "💡 Neo4j status check (showing last 3 log lines):"
            docker logs spotify-neo4j --tail=3
        fi
    done
    
    echo "❌ Neo4j failed to start within $timeout seconds"
    echo "💡 Final container logs:"
    docker logs spotify-neo4j --tail=20
    echo ""
    echo "🔧 Troubleshooting steps:"
    echo "   1. Check Docker memory/disk space: docker system df"
    echo "   2. Clean up: docker system prune -f --volumes"
    echo "   3. Restart Docker service"
    echo "   4. Check Neo4j configuration in docker-compose.yml"
    exit 1
}

dev_mode() {
    echo "🚀 Starting development mode..."
    
    install_tools
    start_services
    
    # Create .env if it doesn't exist
    # Create .env if it doesn't exist
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "📄 Created .env file from template"
        echo "⚠️  Please edit .env file with your API credentials"
        echo "💡 Tip: Use './setup_apis.sh' for guided API configuration"
    fi
    
    # Check API configuration
    echo "🔍 Checking API Configuration..."
    source .env
    if [ -n "$SPOTIFY_CLIENT_ID" ] && [ "$SPOTIFY_CLIENT_ID" != "your_spotify_client_id_here" ]; then
        echo "✅ Spotify API: Configured"
        # Try to refresh Spotify token before starting
        echo "🔄 Refreshing Spotify access token..."
        ./get_spotify_token.sh || echo "⚠️  Token refresh failed, continuing..."
    else
        echo "⚠️  Spotify API: Not configured - some features may not work"
        echo "   Run './setup_apis.sh' to configure APIs"
    fi
    
    echo ""
    echo "🎯 Development servers will start:"
    echo "   📱 Frontend (WebAssembly): http://localhost:8080"
    echo "   🔧 Backend API: http://localhost:3000"
    echo "   🗄️  Neo4j Browser: http://localhost:7474 (neo4j/password123)"
    echo ""
    echo "Starting backend server in background..."
    cd backend
    cargo run &
    BACKEND_PID=$!
    cd ..
    
    # Wait a moment for backend to start
    sleep 3
    
    echo "Starting frontend development server..."
    cd frontend
    trunk serve --open
    
    # Cleanup when script exits
    trap "kill $BACKEND_PID 2>/dev/null" EXIT
}

prod_mode() {
    echo "🏭 Building for production..."
    
    install_tools
    build_frontend
    build_backend
    
    echo ""
    echo "✅ Production build complete!"
    echo "📁 Frontend files: frontend/dist/"
    echo "📁 Backend binary: backend/target/release/spotify-neo4j-backend"
    echo ""
    echo "To run in production:"
    echo "  1. Start Neo4j: docker compose up -d neo4j"
    echo "  2. Set up .env file with production API keys"
    echo "  3. Run: ./backend/target/release/spotify-neo4j-backend"
    echo "  4. Access: http://localhost:3000"
}

case $MODE in
    "dev"|"development")
        dev_mode
        ;;
    "build"|"prod"|"production")
        prod_mode
        ;;
    "install")
        install_tools
        ;;
    "cleanup"|"clean")
        cleanup_docker
        ;;
    *)
        echo "Usage: $0 [dev|build|install|cleanup]"
        echo ""
        echo "Commands:"
        echo "  dev     - Start development servers (default)"
        echo "  build   - Build for production"
        echo "  install - Install required tools"
        echo "  cleanup - Clean up Docker containers and resources"
        echo ""
        echo "Examples:"
        echo "  $0           # Start development mode"
        echo "  $0 dev       # Start development mode"
        echo "  $0 build     # Build for production"
        echo "  $0 install   # Install tools only"
        echo "  $0 cleanup   # Clean up Docker resources"
        ;;
esac
