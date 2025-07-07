#!/bin/bash

# Command line arguments
QUIET_MODE=false
FORCE_REFRESH=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -q|--quiet)
            QUIET_MODE=true
            shift
            ;;
        -f|--force)
            FORCE_REFRESH=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -q, --quiet    Suppress output messages (useful for automation)"
            echo "  -f, --force    Force token refresh even if current token is valid"
            echo "  -h, --help     Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0              # Get token with full output"
            echo "  $0 --quiet      # Get token silently"
            echo "  $0 --force      # Force refresh current token"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to log messages (respects quiet mode)
log_message() {
    if [ "$QUIET_MODE" = false ]; then
        echo "$1"
    fi
}

# Function to create .env file from template if it doesn't exist
create_env_file() {
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            log_message "📄 Creating .env file from template..."
            cp .env.example .env
            log_message "⚠️  Please edit .env file with your Spotify API credentials"
            log_message "   Get your credentials from: https://developer.spotify.com/dashboard"
            log_message ""
        else
            log_message "❌ Error: .env.example file not found"
            log_message "   Please create a .env file with your Spotify credentials"
            exit 1
        fi
    fi
}

# Function to validate credentials
validate_credentials() {
    if [ -z "$SPOTIFY_CLIENT_ID" ] || [ "$SPOTIFY_CLIENT_ID" = "your_spotify_client_id_here" ]; then
        log_message "❌ Error: SPOTIFY_CLIENT_ID not configured"
        log_message "   Please edit .env file and set your Spotify Client ID"
        exit 1
    fi

    if [ -z "$SPOTIFY_CLIENT_SECRET" ] || [ "$SPOTIFY_CLIENT_SECRET" = "your_spotify_client_secret_here" ]; then
        log_message "❌ Error: SPOTIFY_CLIENT_SECRET not configured"
        log_message "   Please edit .env file and set your Spotify Client Secret"
        exit 1
    fi
}

# Function to check if current token is valid (only if not forcing refresh)
check_current_token() {
    if [ "$FORCE_REFRESH" = true ]; then
        return 1  # Force refresh
    fi
    
    if [ -n "$SPOTIFY_ACCESS_TOKEN" ] && [ "$SPOTIFY_ACCESS_TOKEN" != "your_spotify_access_token_here" ]; then
        # Test the token with a simple API call
        response=$(curl -s -H "Authorization: Bearer $SPOTIFY_ACCESS_TOKEN" "https://api.spotify.com/v1/me")
        if ! echo "$response" | grep -q "error"; then
            log_message "✅ Current token is still valid"
            return 0  # Token is valid
        fi
    fi
    
    return 1  # Token is invalid or expired
}

# Create .env file if it doesn't exist
create_env_file

# Load environment variables
source .env

# Validate credentials before making API call
validate_credentials

# Check current token validity
if check_current_token; then
    exit 0
fi

# Get Spotify access token using Client Credentials flow
log_message "🎵 Getting Spotify access token..."

RESPONSE=$(curl -s -X POST "https://accounts.spotify.com/api/token" \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=client_credentials&client_id=$SPOTIFY_CLIENT_ID&client_secret=$SPOTIFY_CLIENT_SECRET")

# Extract access token from response using basic text processing
ACCESS_TOKEN=$(echo $RESPONSE | sed 's/.*"access_token":"\([^"]*\)".*/\1/')

# Check if we got a valid response
if [ -z "$RESPONSE" ]; then
    log_message "❌ Error: No response from Spotify API"
    log_message "   Please check your internet connection and try again"
    exit 1
fi

# Check for API error responses
if echo "$RESPONSE" | grep -q "error"; then
    log_message "❌ Error: Spotify API returned an error"
    log_message "   Response: $RESPONSE"
    log_message "   Please check your Client ID and Client Secret in .env file"
    exit 1
fi

if [ "$ACCESS_TOKEN" != "" ] && [[ "$ACCESS_TOKEN" =~ ^BQ ]]; then
    log_message "✅ Access token obtained successfully!"
    if [ "$QUIET_MODE" = false ]; then
        echo "🔑 ACCESS_TOKEN=$ACCESS_TOKEN"
    fi
    log_message ""
    log_message "⏰ Note: This token expires in 1 hour. You'll need to regenerate it periodically."
    log_message ""
    
    # Automatically add to .env file
    if grep -q "SPOTIFY_ACCESS_TOKEN=" .env; then
        sed -i "s/SPOTIFY_ACCESS_TOKEN=.*/SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN/" .env
        log_message "✅ Updated SPOTIFY_ACCESS_TOKEN in .env file"
    else
        echo "SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN" >> .env
        log_message "✅ Added SPOTIFY_ACCESS_TOKEN to .env file"
    fi
    
    log_message ""
    log_message "🎉 Spotify token setup complete!"
    log_message "   Your backend can now access Spotify API"
else
    log_message "❌ Failed to get valid access token"
    log_message "   Expected token starting with 'BQ' but got: $ACCESS_TOKEN"
    log_message "   Full response: $RESPONSE"
    log_message ""
    log_message "💡 Troubleshooting:"
    log_message "   1. Check your SPOTIFY_CLIENT_ID and SPOTIFY_CLIENT_SECRET in .env"
    log_message "   2. Verify credentials at https://developer.spotify.com/dashboard"
    log_message "   3. Ensure your app has the correct permissions"
    exit 1
fi
