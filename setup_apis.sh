#!/bin/bash

echo "🚀 Spotify Neo4j API Setup Wizard"
echo "================================="
echo ""

# Function to create .env file from template if it doesn't exist
create_env_file() {
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            echo "📄 Creating .env file from template..."
            cp .env.example .env
            echo "✅ Created .env file"
        else
            echo "❌ Error: .env.example file not found"
            exit 1
        fi
    else
        echo "📄 Using existing .env file"
    fi
    echo ""
}

# Function to setup Spotify API credentials
setup_spotify() {
    echo "🎵 Spotify API Setup"
    echo "-------------------"
    echo ""
    echo "To get your Spotify API credentials:"
    echo "1. Go to https://developer.spotify.com/dashboard"
    echo "2. Log in with your Spotify account"
    echo "3. Create a new app or select an existing one"
    echo "4. Copy the Client ID and Client Secret"
    echo ""
    
    read -p "Enter your Spotify Client ID: " spotify_client_id
    read -p "Enter your Spotify Client Secret: " spotify_client_secret
    
    if [ -n "$spotify_client_id" ] && [ -n "$spotify_client_secret" ]; then
        # Update .env file with new credentials
        sed -i "s/SPOTIFY_CLIENT_ID=.*/SPOTIFY_CLIENT_ID=$spotify_client_id/" .env
        sed -i "s/SPOTIFY_CLIENT_SECRET=.*/SPOTIFY_CLIENT_SECRET=$spotify_client_secret/" .env
        echo "✅ Spotify credentials saved to .env file"
        
        # Automatically get access token
        echo ""
        echo "🔄 Getting Spotify access token..."
        ./get_spotify_token.sh
        
        if [ $? -eq 0 ]; then
            echo "✅ Spotify API setup complete!"
        else
            echo "❌ Failed to get Spotify access token. Please check your credentials."
            return 1
        fi
    else
        echo "❌ Invalid credentials provided"
        return 1
    fi
    echo ""
}

# Function to setup YouTube API credentials
setup_youtube() {
    echo "📺 YouTube API Setup"
    echo "-------------------"
    echo ""
    echo "To get your YouTube API key:"
    echo "1. Go to https://console.cloud.google.com/"
    echo "2. Create a new project or select an existing one"
    echo "3. Enable the YouTube Data API v3"
    echo "4. Create credentials (API key)"
    echo ""
    
    read -p "Enter your YouTube API key (or press Enter to skip): " youtube_api_key
    
    if [ -n "$youtube_api_key" ]; then
        # Update .env file with new API key
        sed -i "s/YOUTUBE_API_KEY=.*/YOUTUBE_API_KEY=$youtube_api_key/" .env
        echo "✅ YouTube API key saved to .env file"
        echo ""
    else
        echo "⏭️  Skipping YouTube API setup"
        echo ""
    fi
}

# Function to validate all API configurations
validate_setup() {
    echo "🔍 Validating API Setup"
    echo "---------------------"
    echo ""
    
    source .env
    
    # Check Spotify configuration
    if [ -n "$SPOTIFY_CLIENT_ID" ] && [ "$SPOTIFY_CLIENT_ID" != "your_spotify_client_id_here" ] && \
       [ -n "$SPOTIFY_CLIENT_SECRET" ] && [ "$SPOTIFY_CLIENT_SECRET" != "your_spotify_client_secret_here" ] && \
       [ -n "$SPOTIFY_ACCESS_TOKEN" ] && [ "$SPOTIFY_ACCESS_TOKEN" != "your_spotify_access_token_here" ]; then
        echo "✅ Spotify API: Configured"
    else
        echo "⚠️  Spotify API: Not fully configured"
    fi
    
    # Check YouTube configuration
    if [ -n "$YOUTUBE_API_KEY" ] && [ "$YOUTUBE_API_KEY" != "your_youtube_api_key_here" ]; then
        echo "✅ YouTube API: Configured"
    else
        echo "⚠️  YouTube API: Not configured"
    fi
    
    # Check Neo4j configuration
    if [ -n "$NEO4J_URI" ] && [ -n "$NEO4J_USER" ] && [ -n "$NEO4J_PASSWORD" ]; then
        echo "✅ Neo4j Database: Configured"
    else
        echo "⚠️  Neo4j Database: Not configured"
    fi
    
    echo ""
}

# Main setup flow
echo "This wizard will help you configure API credentials for the Spotify Neo4j application."
echo ""

# Create .env file
create_env_file

# Setup APIs
echo "Which APIs would you like to configure?"
echo "1. Spotify API (required for core functionality)"
echo "2. YouTube API (optional, for playlist creation)"
echo "3. Both APIs"
echo "4. Skip API setup"
echo ""

read -p "Enter your choice (1-4): " choice

case $choice in
    1)
        setup_spotify
        ;;
    2)
        setup_youtube
        ;;
    3)
        setup_spotify
        setup_youtube
        ;;
    4)
        echo "⏭️  Skipping API setup"
        ;;
    *)
        echo "❌ Invalid choice. Please run the script again."
        exit 1
        ;;
esac

# Validate final setup
validate_setup

echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Start the development servers with: ./dev.sh"
echo "2. Or run the backend directly with: cd backend && cargo run"
echo "3. Access the application at: http://localhost:3000"
echo ""
echo "💡 Tips:"
echo "- Run './get_spotify_token.sh' to refresh your Spotify token (expires in 1 hour)"
echo "- Use './setup_apis.sh' again to update your API credentials"
echo "- Check the README.md for more detailed setup instructions"