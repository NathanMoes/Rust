# Docker Container Troubleshooting Guide

## Issues Fixed

### Problem: Neo4j Container Failing to Start Consistently

**Symptoms:**
- Container exits with code 1
- Container keeps restarting
- Error logs showing configuration issues

**Root Causes Identified:**
1. **Invalid Neo4j configuration**: `db.logs.query.enabled=false` should be `OFF`
2. **Conflicting container states**: Old containers not properly cleaned up
3. **Resource constraints**: Insufficient memory allocation
4. **Plugin configuration**: APOC plugin security settings

### Solutions Implemented

#### 1. Fixed Docker Compose Configuration
```yaml
environment:
  - NEO4J_db_logs_query_enabled=OFF  # Was: false (invalid)
  - NEO4J_server_memory_heap_max__size=1G  # Reduced from 2G
  - NEO4J_dbms_security_procedures_unrestricted=apoc.*  # Added
  - NEO4J_server_memory_pagecache_size=256m  # Added
```

#### 2. Enhanced Development Script (`dev.sh`)

**Improved container management:**
- Automatic cleanup of failed containers
- Better error handling and retry logic
- Extended timeout periods (180s vs 120s)
- Progressive health checking with detailed logging

**Added cleanup command:**
```bash
./dev.sh cleanup  # Clean up Docker resources
```

#### 3. Health Check Script
Created `health_check.sh` for diagnosing issues:
```bash
./health_check.sh  # Comprehensive system health check
```

## Troubleshooting Commands

### Quick Fixes
```bash
# Clean up and restart
./dev.sh cleanup
./dev.sh dev

# Check container status
docker ps -a
docker logs spotify-neo4j --tail=20

# Test connectivity
docker exec spotify-neo4j cypher-shell -u neo4j -p password123 "RETURN 1;"
```

### System Cleanup
```bash
# Remove all stopped containers
docker container prune -f

# Remove unused volumes (CAUTION: This removes data!)
docker volume prune -f

# Full system cleanup
docker system prune -f --volumes
```

### Memory Issues
```bash
# Check Docker memory usage
docker system df

# Reduce Neo4j memory if needed (edit docker-compose.yml):
NEO4J_server_memory_heap_max__size=512m
NEO4J_server_memory_pagecache_size=128m
```

## Prevention

1. **Always use the dev script**: `./dev.sh dev` instead of manual Docker commands
2. **Regular cleanup**: Run `./dev.sh cleanup` when switching between projects
3. **Monitor resources**: Check `docker system df` periodically
4. **Use health checks**: Run `./health_check.sh` to verify system state

## Updated Development Workflow

```bash
# Start development (automatically handles cleanup if needed)
./dev.sh dev

# If issues persist
./dev.sh cleanup
./health_check.sh
./dev.sh dev

# Production build
./dev.sh build
```

## Configuration Best Practices

1. **Neo4j Environment Variables**: Use correct value types (OFF/INFO/VERBOSE vs true/false)
2. **Memory Allocation**: Start with conservative values, increase if needed
3. **Health Checks**: Always include proper health check configuration
4. **Restart Policy**: Use `unless-stopped` for development containers
5. **Volume Management**: Separate data, logs, and plugins for easier debugging
