#!/bin/bash

echo "🏥 Health Check for Spotify Neo4j Development Environment"
echo "========================================================"

# Check Docker service
echo "🐳 Checking Docker service..."
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running or accessible"
    exit 1
fi
echo "✅ Docker is running"

# Check Neo4j container
echo ""
echo "📊 Checking Neo4j container..."
if ! docker ps | grep -q spotify-neo4j; then
    echo "❌ Neo4j container is not running"
    echo "💡 Try running: ./dev.sh cleanup && ./dev.sh dev"
    exit 1
fi

# Get container status
CONTAINER_STATUS=$(docker inspect --format='{{.State.Status}}' spotify-neo4j 2>/dev/null)
CONTAINER_HEALTH=$(docker inspect --format='{{.State.Health.Status}}' spotify-neo4j 2>/dev/null)

echo "✅ Neo4j container is running"
echo "   Status: $CONTAINER_STATUS"
echo "   Health: $CONTAINER_HEALTH"

# Test Neo4j connectivity
echo ""
echo "🔗 Testing Neo4j connectivity..."
if docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "RETURN 1;" >/dev/null 2>&1; then
    echo "✅ Neo4j is responding to queries"
else
    echo "❌ Neo4j is not responding"
    echo "💡 Container logs:"
    docker logs spotify-neo4j --tail=10
    exit 1
fi

# Check ports
echo ""
echo "🌐 Checking port accessibility..."
if curl -s http://localhost:7474 >/dev/null; then
    echo "✅ Neo4j Browser accessible at http://localhost:7474"
else
    echo "⚠️  Neo4j Browser may not be accessible at http://localhost:7474"
fi

# Check Docker system health
echo ""
echo "💾 Docker system status:"
docker system df

echo ""
echo "🎉 Health check complete!"
echo ""
echo "🔗 Useful URLs:"
echo "   Neo4j Browser: http://localhost:7474 (neo4j/password123)"
echo "   Backend API: http://localhost:3000 (when running)"
echo "   Frontend: http://localhost:8080 (when running)"
