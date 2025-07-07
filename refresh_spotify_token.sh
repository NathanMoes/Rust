#!/bin/bash

# Spotify Token Refresh Utility
# This script can be called by the backend to refresh tokens automatically

# Function to check if token is still valid
check_token_validity() {
    local token=$1
    
    if [ -z "$token" ] || [ "$token" = "your_spotify_access_token_here" ]; then
        return 1
    fi
    
    # Test the token by making a simple API call
    response=$(curl -s -H "Authorization: Bearer $token" "https://api.spotify.com/v1/me")
    
    if echo "$response" | grep -q "error"; then
        return 1
    fi
    
    return 0
}

# Function to refresh token and update .env
refresh_token() {
    local quiet_mode=$1
    
    if [ "$quiet_mode" != "quiet" ]; then
        echo "🔄 Refreshing Spotify access token..."
    fi
    
    # Source environment variables
    if [ -f .env ]; then
        source .env
    else
        if [ "$quiet_mode" != "quiet" ]; then
            echo "❌ Error: .env file not found"
        fi
        return 1
    fi
    
    # Check if credentials are configured
    if [ -z "$SPOTIFY_CLIENT_ID" ] || [ "$SPOTIFY_CLIENT_ID" = "your_spotify_client_id_here" ] || \
       [ -z "$SPOTIFY_CLIENT_SECRET" ] || [ "$SPOTIFY_CLIENT_SECRET" = "your_spotify_client_secret_here" ]; then
        if [ "$quiet_mode" != "quiet" ]; then
            echo "❌ Error: Spotify credentials not configured"
        fi
        return 1
    fi
    
    # Get new access token
    RESPONSE=$(curl -s -X POST "https://accounts.spotify.com/api/token" \
         -H "Content-Type: application/x-www-form-urlencoded" \
         -d "grant_type=client_credentials&client_id=$SPOTIFY_CLIENT_ID&client_secret=$SPOTIFY_CLIENT_SECRET")
    
    # Extract access token from response
    ACCESS_TOKEN=$(echo $RESPONSE | sed 's/.*"access_token":"\([^"]*\)".*/\1/')
    
    if [ "$ACCESS_TOKEN" != "" ] && [[ "$ACCESS_TOKEN" =~ ^BQ ]]; then
        # Update .env file
        if grep -q "SPOTIFY_ACCESS_TOKEN=" .env; then
            sed -i "s/SPOTIFY_ACCESS_TOKEN=.*/SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN/" .env
        else
            echo "SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN" >> .env
        fi
        
        if [ "$quiet_mode" != "quiet" ]; then
            echo "✅ Token refreshed successfully"
        fi
        
        # Output the new token for scripts that need it
        echo "$ACCESS_TOKEN"
        return 0
    else
        if [ "$quiet_mode" != "quiet" ]; then
            echo "❌ Failed to refresh token"
        fi
        return 1
    fi
}

# Function to get current token or refresh if needed
get_valid_token() {
    local quiet_mode=$1
    
    # Source environment variables
    if [ -f .env ]; then
        source .env
    else
        if [ "$quiet_mode" != "quiet" ]; then
            echo "❌ Error: .env file not found"
        fi
        return 1
    fi
    
    # Check if current token is valid
    if check_token_validity "$SPOTIFY_ACCESS_TOKEN"; then
        if [ "$quiet_mode" != "quiet" ]; then
            echo "✅ Current token is valid"
        fi
        echo "$SPOTIFY_ACCESS_TOKEN"
        return 0
    else
        if [ "$quiet_mode" != "quiet" ]; then
            echo "🔄 Current token is invalid or expired, refreshing..."
        fi
        refresh_token "$quiet_mode"
        return $?
    fi
}

# Main script logic
case "${1:-get}" in
    "check")
        source .env 2>/dev/null || exit 1
        if check_token_validity "$SPOTIFY_ACCESS_TOKEN"; then
            echo "✅ Token is valid"
            exit 0
        else
            echo "❌ Token is invalid or expired"
            exit 1
        fi
        ;;
    "refresh")
        refresh_token "${2:-}"
        ;;
    "get")
        get_valid_token "${2:-}"
        ;;
    *)
        echo "Usage: $0 [check|refresh|get] [quiet]"
        echo ""
        echo "Commands:"
        echo "  check   - Check if current token is valid"
        echo "  refresh - Force refresh the token"
        echo "  get     - Get valid token (refresh if needed)"
        echo ""
        echo "Options:"
        echo "  quiet   - Suppress output messages"
        exit 1
        ;;
esac