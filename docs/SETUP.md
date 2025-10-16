# Setup and Installation Guide

## Prerequisites

### Backend Requirements
- Rust 1.75 or higher
- Cargo package manager
- OpenSSL development libraries

### Frontend Requirements
- Node.js 18 or higher
- npm 9 or higher

## Installation Steps

### 1. Clone Repository
```bash
git clone <repository-url>
cd raiku-simulator-v2
```

### 2. Backend Setup
```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build project
cargo build --release

# Run tests
cargo test

# Start server
cargo run
```

Backend will start at `http://localhost:8080`

### 3. Frontend Setup
```bash
# Navigate to frontend directory
cd raiku-frontend

# Install dependencies
npm install

# Start development server
npm start
```

Frontend will start at `http://localhost:3000`

## Configuration

Create `.env` file in backend root:
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CORS_ORIGINS=http://localhost:3000
SLOT_DURATION_MS=400
BASE_FEE_SOL=0.001
ADVANCE_SLOT_INTERVAL_MS=400
AOT_DURATION_SEC=35
```

### Configuration Parameters

**SERVER_HOST**: Interface to bind (0.0.0.0 for all interfaces)
**SERVER_PORT**: HTTP port number
**CORS_ORIGINS**: Comma-separated allowed origins
**SLOT_DURATION_MS**: Milliseconds per slot
**BASE_FEE_SOL**: Minimum fee per slot in SOL
**ADVANCE_SLOT_INTERVAL_MS**: Time between slot advancements
**AOT_DURATION_SEC**: Default AOT auction duration

## Docker Deployment
```bash
# Build image
docker build -t raiku-simulator .

# Run container
docker run -p 8080:8080 \
  -e SERVER_HOST=0.0.0.0 \
  -e SERVER_PORT=8080 \
  raiku-simulator
```

## Verification

### Backend Health Check
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "success": true,
  "message": "Server is healthy.",
  "data": {
    "status": "healthy",
    "timestamp": "2025-01-15T10:30:00Z"
  },
  "code": 200
}
```

### Frontend Access

Navigate to `http://localhost:3000` in browser. Should see:
- Raiku Simulator header
- Player stats panel
- Slot marketplace grid
- Tab navigation

### API Documentation

Navigate to `http://localhost:8080/swagger-ui` to verify OpenAPI documentation loads.