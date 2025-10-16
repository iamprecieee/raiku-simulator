# Raiku Slot Auction Simulator - Hackathon Submission

**Track**: Visual Simulations & Blueprints  
**Team**: iamprecieee  
**Submission Type**: Interactive Prototype

## Executive Summary

The Raiku Slot Auction Simulator is a full-stack(backend-focused) application that demonstrates how Raiku's deterministic execution model works through interactive participation in JIT and AOT auctions. Users can experience guaranteed slot inclusion firsthand while developers gain reference patterns for integration.

## Problem Context

Raiku addresses critical infrastructure problems on Solana:

**Network Congestion**: Between January and February 2025, Solana experienced 35-40% revert rates for non-vote transactions. Addresses conducting over 10,000 daily transactions saw revert rates of 66.7%, with high-activity addresses responsible for 95.2% of all reverted transactions.

**MEV Extraction Spam**: A significant portion of Solana's transaction volume consists of spam from MEV extraction attempts. Bots compete for arbitrage opportunities by flooding the network with transactions, most of which fail.

**Execution Uncertainty**: Applications cannot guarantee transaction inclusion during congestion, making it impossible to build reliable financial infrastructure that requires deterministic outcomes.

## Solution Approach

This simulator demonstrates Raiku's solution through two key mechanisms:

### 1. Interactive Slot Marketplace

A visual grid of 100 rolling slots where users can observe state transitions in real-time. Each slot represents discrete blockspace with defined compute unit capacity (48M CU per slot). Users see exactly when slots become available, enter auction states, get reserved, or expire.

### 2. Dual Auction Mechanics

**JIT Auctions**: First-price sealed-bid auctions for immediate execution. Users bid for the next available slot without seeing competitor bids. Highest bidder wins and pays their bid amount. 

**AOT Auctions**: English-style auctions for future slots (35+ slots ahead). Users bid openly during a defined period with transparent price discovery. Losing bidders receive automatic refunds.

## Technical Implementation

### Backend Architecture

**Core Components**:
- State management through Arc<RwLock> for thread-safe concurrent access
- Auction manager handling bid submission and resolution logic
- Session manager with cookie-based authentication
- Event broadcaster using tokio channels for SSE

**Key Design Decisions**:
- Separation of concerns: managers handle business logic, routes handle HTTP
- Type-safe enums for transaction and slot states
- Automatic refund processing for losing AOT bidders
- Rate limiting per client IP to prevent abuse

**Economic Model**:
- Players start with 100,000 SOL balance
- Bids deducted immediately upon submission
- Winning bidders pay their bid amount
- Losing AOT bidders receive automatic refunds
- Transaction fees calculated per slot

### Frontend Architecture

**Interface Components**:
- Slot grid showing real-time state changes
- Auction panels for JIT and AOT participation
- Transaction history with pagination
- Player stats dashboard with progression system
- Global leaderboard across three categories

**Real-time Updates**:
- Server-Sent Events connection for live data
- Automatic reconnection on connection loss
- Notification system for auction results
- Achievement popup system

### Integration Patterns

The codebase demonstrates production patterns:

**Session Management**: Cookie-based sessions with validation middleware showing how to maintain user state across requests.

**Balance Checking**: Pre-flight balance validation before accepting bids, with proper error responses when insufficient funds exist.

**Concurrent Bid Handling**: Multiple users can bid simultaneously on different slots without race conditions due to locking strategies.

**Refund Processing**: Automatic balance restoration for losing bidders, grouped by player to minimize state updates.

## Deliverables

### 1. Working Prototype
Full-stack application with Rust backend and React frontend. All core functionality implemented: session creation, auction participation, transaction tracking, real-time updates, player progression.

### 2. API Documentation
OpenAPI 3.0 specification with interactive Swagger UI. Every endpoint documented with request schemas, response formats, and error codes. Developers can test all endpoints directly from the browser.

### 3. Economic Validation
Proper auction mechanics with sealed-bid for JIT and English-style for AOT. Automatic refund processing validates the economic model works correctly under concurrent usage.

## Use Case Demonstrations

### Institutional Settlements
AOT auctions show how institutions can reserve specific future slots for scheduled operations. The English auction format allows for efficient price discovery while planning reduces costs versus urgent JIT bids.

### DeFi Liquidations
JIT auctions prove reliable execution for time-critical liquidations. The guarantee of inclusion within 1-2 seconds prevents bad debt accumulation from failed liquidation attempts.

## Innovation Highlights

### Educational Gamification
The progression system teaches auction mechanics through experiential learning. Players naturally discover optimal bidding strategies, understand timing tradeoffs, and experience guaranteed execution firsthand.

### Real Economic Model
Unlike theoretical demonstrations, this simulator implements economic consequences. Bid amounts affect balance, losses trigger refunds, and competition emerges naturally as multiple users participate.

### Visual State Machine
The slot grid provides immediate visual feedback on state transitions. Users see exactly how slots move from available to auctioned to reserved to filled, building intuition about the marketplace dynamics.

## Relevance to Raiku's Mission

This simulator directly addresses Raiku's goal of making Solana execution deterministic and reliable:

**Demonstrates Core Primitives**: JIT and AOT transactions are not abstract concepts but interactive experiences users can try immediately.

**Validates Economic Model**: The auction mechanics prove that a fair marketplace for blockspace can exist with proper price discovery and refund handling.

**Proves Scalability**: The concurrent auction handling and real-time state updates demonstrate that the model works well under multi-user load.

## Conclusion

This simulator makes Raiku's deterministic execution model tangible through interactive experience. The visual demonstration serves as an educational tool for understanding guaranteed execution on Raiku.