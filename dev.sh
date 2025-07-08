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
    
    # Clean up unused Docker resources (but preserve volumes by default)
    echo "Cleaning up unused Docker resources (preserving data volumes)..."
    docker system prune -f
    
    # Remove volumes if requested
    echo ""
    echo "💾 Your Neo4j data is preserved in Docker volumes."
    read -p "🗑️  Do you want to PERMANENTLY DELETE all Neo4j data? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "⚠️  Removing Neo4j data volumes..."
        docker volume rm rust_neo4j_data rust_neo4j_logs rust_neo4j_import rust_neo4j_plugins 2>/dev/null || true
        docker compose -f docker-compose.yml down -v
        echo "🗑️  All Neo4j data has been deleted!"
    else
        echo "✅ Neo4j data volumes preserved."
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
        echo "🔧 Attempting to clean up and retry (preserving data volumes)..."
        docker system prune -f  # Removed --volumes flag to preserve data
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

check_data() {
    echo "💾 Neo4j Data Status"
    echo "==================="
    
    # Check for volumes
    echo "📊 Docker Volumes:"
    if docker volume ls | grep -q "rust_neo4j"; then
        docker volume ls | grep "rust_neo4j" | while read driver name; do
            size=$(docker system df -v | grep "$name" | awk '{print $3}' || echo "Unknown")
            echo "   ✅ $name (Size: $size)"
        done
    else
        echo "   ❌ No Neo4j data volumes found"
    fi
    
    # Check container status
    echo ""
    echo "🐳 Container Status:"
    if docker ps -a | grep -q spotify-neo4j; then
        status=$(docker inspect --format='{{.State.Status}}' spotify-neo4j 2>/dev/null)
        echo "   ✅ spotify-neo4j container exists (Status: $status)"
        
        if [ "$status" = "running" ]; then
            echo ""
            echo "🔍 Database Info:"
            if docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "MATCH (n) RETURN count(n) as total_nodes;" 2>/dev/null; then
                echo "   ✅ Database is accessible"
            else
                echo "   ⚠️  Database not responding"
            fi
        fi
    else
        echo "   ❌ No spotify-neo4j container found"
    fi
    
    echo ""
    echo "💡 Commands:"
    echo "   ./dev.sh dev     - Start development (preserves data)"
    echo "   ./dev.sh cleanup - Clean containers (ask about data)"
    echo "   ./health_check.sh - Full system health check"
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
    "data"|"status")
        check_data
        ;;
    *)
        echo "Usage: $0 [dev|build|install|cleanup|data]"
        echo ""
        echo "Commands:"
        echo "  dev     - Start development servers (default)"
        echo "  build   - Build for production"
        echo "  install - Install required tools"
        echo "  cleanup - Clean up Docker containers and resources"
        echo "  data    - Check Neo4j data status"
        echo ""
        echo "Examples:"
        echo "  $0           # Start development mode"
        echo "  $0 dev       # Start development mode"
        echo "  $0 build     # Build for production"
        echo "  $0 install   # Install tools only"
        echo "  $0 cleanup   # Clean up Docker resources"
        echo "  $0 data      # Check what data exists"
        ;;
esac
