#!/bin/bash

echo "🎵 Building Spotify Neo4j Frontend (WebAssembly)"
echo "================================================="

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "❌ Trunk is not installed. Installing..."
    cargo install trunk
fi

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then 
    echo "❌ wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Add wasm32 target if not present
rustup target add wasm32-unknown-unknown

echo "🔨 Building WebAssembly frontend..."
cd frontend

# Build the project
trunk build --release

echo "✅ Frontend build complete!"
echo ""
echo "📁 Built files are in: frontend/dist/"
echo "🌐 To serve locally, run: trunk serve"
echo "   or serve the dist/ folder with any static file server"
echo ""
echo "Development mode:"
echo "  cd frontend && trunk serve"
echo ""
