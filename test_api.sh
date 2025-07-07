#!/bin/bash

# Example script to test the Spotify Neo4j Backend
# Make sure to replace the tokens and IDs with your actual values

BASE_URL="http://localhost:3000"

echo "🔍 Testing Health Check..."
curl -s "$BASE_URL/" | jq .

echo -e "\n📥 Testing Spotify Import..."
# Replace with your actual Spotify playlist ID and access token
SPOTIFY_TOKEN="YOUR_SPOTIFY_ACCESS_TOKEN"
PLAYLIST_ID="YOUR_SPOTIFY_PLAYLIST_ID"

curl -s -X POST "$BASE_URL/spotify/import" \
  -H "Content-Type: application/json" \
  -d "{
    \"playlist_id\": \"$PLAYLIST_ID\",
    \"access_token\": \"$SPOTIFY_TOKEN\"
  }" | jq .

echo -e "\n🎤 Getting Artists..."
curl -s "$BASE_URL/spotify/artists" | jq '. | length'

echo -e "\n🎵 Getting Tracks..."
curl -s "$BASE_URL/spotify/tracks" | jq '. | length'

echo -e "\n🤖 Getting Recommendations..."
# Replace with actual track IDs from your database
SEED_TRACKS="track_id_1,track_id_2"
curl -s "$BASE_URL/recommendations?seed_tracks=$SEED_TRACKS&limit=5" | jq .

echo -e "\n📺 Creating YouTube Playlist from Recommendations..."
# Replace with your YouTube access token
YOUTUBE_TOKEN="YOUR_YOUTUBE_ACCESS_TOKEN"

curl -s -X POST "$BASE_URL/youtube/playlist/from-recommendations" \
  -H "Content-Type: application/json" \
  -d "{
    \"seed_tracks\": [\"$SEED_TRACKS\"],
    \"playlist_name\": \"AI Generated Test Playlist\",
    \"youtube_access_token\": \"$YOUTUBE_TOKEN\",
    \"limit\": 10
  }" | jq .

echo -e "\n✅ All tests completed!"
