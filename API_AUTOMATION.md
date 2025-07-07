# API Automation Documentation

This document describes the enhanced API automation features for the Spotify Neo4j application.

## Overview

The application now includes several automated tools to simplify API configuration and token management:

1. **Enhanced Token Management**: Automatic token generation, validation, and persistence
2. **Setup Wizard**: Interactive configuration of API credentials
3. **Backend Integration**: Automatic token refresh and persistence
4. **Command Line Tools**: Flexible scripts for various automation scenarios

## Scripts Overview

### 1. `get_spotify_token.sh` - Enhanced Token Management

**Purpose**: Get and manage Spotify access tokens with improved error handling and automation features.

**Features**:
- Automatic .env file creation from template
- Credential validation before API calls
- Token validity checking (avoids unnecessary API calls)
- Automatic token persistence to .env file
- Quiet mode for automation
- Force refresh option
- Comprehensive error handling and troubleshooting

**Usage**:
```bash
./get_spotify_token.sh [OPTIONS]

Options:
  -q, --quiet    Suppress output messages (useful for automation)
  -f, --force    Force token refresh even if current token is valid
  -h, --help     Show help message

Examples:
  ./get_spotify_token.sh              # Get token with full output
  ./get_spotify_token.sh --quiet      # Get token silently
  ./get_spotify_token.sh --force      # Force refresh current token
```

### 2. `setup_apis.sh` - Interactive Setup Wizard

**Purpose**: Guide users through API credential configuration with interactive prompts.

**Features**:
- Interactive setup for Spotify and YouTube APIs
- Automatic .env file creation and management
- Credential validation and testing
- Setup status reporting
- Helpful links and instructions

**Usage**:
```bash
./setup_apis.sh

# Follow the interactive prompts to configure:
# 1. Spotify API (required for core functionality)
# 2. YouTube API (optional, for playlist creation)
# 3. Both APIs
# 4. Skip API setup
```

### 3. `refresh_spotify_token.sh` - Token Refresh Utility

**Purpose**: Standalone utility for token management, designed for integration with backend and automation scripts.

**Features**:
- Check token validity
- Force token refresh
- Get valid token (refresh if needed)
- Quiet mode for automation
- Suitable for backend integration

**Usage**:
```bash
./refresh_spotify_token.sh [COMMAND] [OPTIONS]

Commands:
  check   - Check if current token is valid
  refresh - Force refresh the token
  get     - Get valid token (refresh if needed)

Options:
  quiet   - Suppress output messages

Examples:
  ./refresh_spotify_token.sh check     # Check token validity
  ./refresh_spotify_token.sh refresh   # Force refresh
  ./refresh_spotify_token.sh get quiet # Get valid token silently
```

### 4. `test_token_system.sh` - Test Suite

**Purpose**: Comprehensive testing of all token management features.

**Usage**:
```bash
./test_token_system.sh
```

## Backend Integration

### Automatic Token Persistence

The Rust backend now automatically saves generated tokens to the .env file:

```rust
// In spotify.rs
pub async fn get_access_token(&self) -> Result<String> {
    // ... token generation logic ...
    
    // Try to save token to .env file for persistence
    if let Err(e) = self.save_token_to_env(&token_response.access_token).await {
        warn!("Failed to save token to .env file: {}", e);
    }
    
    Ok(token_response.access_token)
}
```

### Token Management Flow

1. **Backend tries to use existing token** from environment variable
2. **If token is missing/invalid**, generates new token via API
3. **Automatically saves new token** to .env file for future use
4. **Logs all token operations** for debugging

## Integration with Existing Scripts

### Enhanced `dev.sh`

The development script now:
- Checks API configuration status
- Automatically refreshes Spotify tokens before starting
- Provides setup guidance for unconfigured APIs

### Enhanced `setup.sh`

The setup script now:
- Validates API configuration
- Automatically refreshes tokens during setup
- Provides integration with the setup wizard

## Configuration File Structure

The .env file structure remains the same but now includes better management:

```bash
# Neo4j Database Configuration
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password123

# Spotify API Configuration
SPOTIFY_CLIENT_ID=your_spotify_client_id_here
SPOTIFY_CLIENT_SECRET=your_spotify_client_secret_here
SPOTIFY_ACCESS_TOKEN=your_spotify_access_token_here  # Auto-managed

# YouTube API Configuration
YOUTUBE_API_KEY=your_youtube_api_key_here

# Server Configuration
BIND_ADDRESS=0.0.0.0:3000
```

## Error Handling and Troubleshooting

### Common Issues and Solutions

1. **Missing Credentials**:
   - Error: "SPOTIFY_CLIENT_ID not configured"
   - Solution: Run `./setup_apis.sh` for guided setup

2. **Invalid Credentials**:
   - Error: "Spotify API returned an error"
   - Solution: Verify credentials at https://developer.spotify.com/dashboard

3. **Network Issues**:
   - Error: "No response from Spotify API"
   - Solution: Check internet connection and try again

4. **Token Validation Failures**:
   - Error: "Token is invalid or expired"
   - Solution: Run `./get_spotify_token.sh --force` to refresh

### Debugging

Enable verbose logging by:
- Using scripts without `--quiet` flag
- Checking backend logs for token operations
- Running `./test_token_system.sh` for comprehensive testing

## Security Considerations

1. **Token Expiration**: Spotify tokens expire in 1 hour
2. **Credential Storage**: Credentials are stored in .env file (not committed to git)
3. **Token Persistence**: Tokens are automatically refreshed and saved
4. **Error Handling**: Sensitive information is not logged in error messages

## Migration Guide

### From Previous Version

If you're upgrading from a previous version:

1. **Backup your .env file**:
   ```bash
   cp .env .env.backup
   ```

2. **Run the setup wizard**:
   ```bash
   ./setup_apis.sh
   ```

3. **Test the new functionality**:
   ```bash
   ./test_token_system.sh
   ```

### New Installation

For new installations:

1. **Run the setup wizard**:
   ```bash
   ./setup_apis.sh
   ```

2. **Start development**:
   ```bash
   ./dev.sh
   ```

## API Reference

### Environment Variables

- `SPOTIFY_CLIENT_ID`: Spotify application Client ID
- `SPOTIFY_CLIENT_SECRET`: Spotify application Client Secret
- `SPOTIFY_ACCESS_TOKEN`: Current Spotify access token (auto-managed)
- `YOUTUBE_API_KEY`: YouTube Data API v3 key
- `NEO4J_URI`: Neo4j database connection string
- `NEO4J_USER`: Neo4j username
- `NEO4J_PASSWORD`: Neo4j password

### Script Exit Codes

All scripts follow standard exit code conventions:
- `0`: Success
- `1`: General error (configuration, API, etc.)
- `2`: Invalid command line arguments

## Future Enhancements

Planned improvements include:
- Token expiration tracking and automatic renewal
- Multi-API token management dashboard
- Integration with CI/CD pipelines
- Enhanced security features
- Monitoring and alerting for token issues