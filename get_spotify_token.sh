#!/bin/bash

# Load environment variables
source .env

# Get Spotify access token using Client Credentials flow
echo "Getting Spotify access token..."

RESPONSE=$(curl -s -X POST "https://accounts.spotify.com/api/token" \
     -H "Content-Type: application/x-www-form-urlencoded" \
     -d "grant_type=client_credentials&client_id=$SPOTIFY_CLIENT_ID&client_secret=$SPOTIFY_CLIENT_SECRET")

# Extract access token from response using basic text processing
ACCESS_TOKEN=$(echo $RESPONSE | sed 's/.*"access_token":"\([^"]*\)".*/\1/')

if [ "$ACCESS_TOKEN" != "" ] && [[ "$ACCESS_TOKEN" =~ ^BQ ]]; then
    echo "Access token obtained successfully!"
    echo "ACCESS_TOKEN=$ACCESS_TOKEN"
    echo ""
    echo "Add this to your .env file:"
    echo "SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN"
    echo ""
    echo "Note: This token expires in 1 hour. You'll need to regenerate it periodically."
    
    # Automatically add to .env file
    if grep -q "SPOTIFY_ACCESS_TOKEN=" .env; then
        sed -i "s/SPOTIFY_ACCESS_TOKEN=.*/SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN/" .env
        echo "Updated SPOTIFY_ACCESS_TOKEN in .env file"
    else
        echo "SPOTIFY_ACCESS_TOKEN=$ACCESS_TOKEN" >> .env
        echo "Added SPOTIFY_ACCESS_TOKEN to .env file"
    fi
else
    echo "Failed to get access token. Response:"
    echo $RESPONSE
fi
