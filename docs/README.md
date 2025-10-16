# Raiku Slot Auction Simulator

A Rust-based simulator demonstrating Raiku's deterministic execution model through interactive JIT and AOT slot auctions. Built for the **Raiku - Inevitable Ideathon**.

## What This Demonstrates

Solana currently suffers from ~35-40% transaction revert rate during congestion periods, with high-activity addresses experiencing revert rates up to 66.7%. Raiku solves this through guaranteed slot inclusion through a marketplace for blockspace.

This simulator provides:
- Gamified interactive experience of JIT and AOT auction mechanics
- Real-time slot marketplace visualization
- Economic model validation with proper bid refunds

## Architecture

**Backend**: Rust + Axum
- RESTful API with OpenAPI documentation
- Server-Sent Events for real-time updates
- Session-based state management
- Rate limiting and CORS handling

**Frontend**: React
- Live slot marketplace grid
- Auction participation interface
- Transaction history tracking
- Player progression system

## Quick Start

### Backend
```bash
cargo run
```
Server runs at `http://localhost:8080`
API documentation at `http://localhost:8080/swagger-ui`

### Frontend
```bash
cd raiku-frontend
npm install
npm start
```
Interface runs at `http://localhost:3000`

## Core Concepts

### Just-in-Time Auctions
First-price sealed-bid auction for immediate slot inclusion. Users bid for the next available slot with execution under 2 seconds. This simulates an environment ideal for time-critical operations like liquidations or MEV opportunities.

### Ahead-of-Time Auctions
English-style auction for future slot reservation (35+ slots ahead). Users bid openly with a defined auction period. Losing bidders receive automatic refunds. This simulates an environment ideal for predictable operations like institutional settlements or scheduled vault rebalancing.

### Slot Marketplace
A rolling window of 100 slots where each slot represents discrete blockspace. Slots transition through states: Available, JIT Auction, AOT Auction, Reserved, Filled, Expired.

## API Overview

All endpoints documented via Swagger UI at `/swagger-ui`

**Session Management**
- `POST /sessions` - Create or validate session

**Marketplace**
- `GET /marketplace/status` - Current marketplace state
- `GET /marketplace/slots` - Available slots
- `GET /marketplace/slots/{slot_number}` - Slot details

**Auctions**
- `GET /auctions/jit` - Active JIT auctions
- `GET /auctions/aot` - Active AOT auctions
- `POST /transactions/jit` - Submit JIT bid
- `POST /transactions/aot` - Submit AOT bid

**Transactions**
- `GET /transactions` - Transaction history
- `GET /transactions/{id}` - Transaction details

**Game Stats**
- `GET /game/player_stats` - Player statistics
- `GET /game/leaderboard` - Global leaderboard

## Environment Configuration
```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CORS_ORIGINS=http://localhost:3000
SLOT_DURATION_MS=400
BASE_FEE_SOL=0.001
ADVANCE_SLOT_INTERVAL_MS=400
AOT_DURATION_SEC=35
```

## Project Structure
```
src/
├── app/              # Application state and API router
├── managers/         # Business logic (auctions, game, sessions)
├── models/           # Data structures and types
├── routes/           # HTTP endpoint handlers
├── services/         # Reusable business services
├── middleware/       # Rate limiting
└── utils/            # Helper functions
```

## Technical Details

- **State Management**: Arc<RwLock> for concurrent access
- **Real-time Updates**: Server-Sent Events
- **Type Safety**: Full Rust type system
- **API Standards**: OpenAPI 3.0 specification
- **Session Handling**: Cookie-based with 24-hour expiration