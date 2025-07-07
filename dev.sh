#!/bin/bash

echo "🎵 Spotify Neo4j Full-Stack Development Setup"
echo "=============================================="

MODE="${1:-dev}"

install_tools() {
    echo "🔧 Installing required tools..."
    
    # Install Trunk for WebAssembly frontend builds
    if ! command -v trunk &> /dev/null; then
        echo "Installing Trunk..."
        cargo install trunk
    fi
    
    # Add wasm32 target
    rustup target add wasm32-unknown-unknown
    
    echo "✅ Tools installed successfully!"
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
    
    # Start Neo4j
    if ! docker ps | grep -q spotify-neo4j; then
        echo "Starting Neo4j database..."
        docker compose up -d neo4j
        
        # Wait for Neo4j to be ready
        echo "⏳ Waiting for Neo4j to be ready..."
        timeout=120
        counter=0
        while ! docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "RETURN 1;" > /dev/null 2>&1; do
            sleep 3
            counter=$((counter + 3))
            if [ $counter -ge $timeout ]; then
                echo "❌ Neo4j failed to start within $timeout seconds"
                echo "💡 Checking Neo4j logs..."
                docker logs spotify-neo4j --tail=20
                exit 1
            fi
            echo "   Still waiting... ($counter/$timeout seconds)"
        done
        echo "✅ Neo4j is ready!"
    else
        echo "✅ Neo4j is already running!"
    fi
}

dev_mode() {
    echo "🚀 Starting development mode..."
    
    install_tools
    start_services
    
    # Create .env if it doesn't exist
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "📄 Created .env file from template"
        echo "⚠️  Please edit .env file with your API credentials"
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
    *)
        echo "Usage: $0 [dev|build|install]"
        echo ""
        echo "Commands:"
        echo "  dev     - Start development servers (default)"
        echo "  build   - Build for production"
        echo "  install - Install required tools"
        echo ""
        echo "Examples:"
        echo "  $0           # Start development mode"
        echo "  $0 dev       # Start development mode"
        echo "  $0 build     # Build for production"
        echo "  $0 install   # Install tools only"
        ;;
esac
