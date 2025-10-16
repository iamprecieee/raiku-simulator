# Architecture Documentation

## System Overview

The Raiku Simulator is a concurrent, stateful application managing multiple user sessions, active auctions, and real-time marketplace updates. The architecture prioritizes correctness under concurrent access, type safety, and clean separation of concerns.


## Core Modules

### App State (app/state.rs)

Central state container using ``` for concurre`nt access. Holds:
- **SlotMarketplace**: rolling window of 100 slots
- **AuctionManager**: active JIT and AOT auctions
- **Transaction store**: global and per-session transaction maps
- **SessionManager**: active user sessions
- **EventBroadcaster**: SSE channel for real-time updates
- **GameManager**: player statistics and progression

All public methods on `AppState` coordinate across these components, ensuring atomic operations and event broadcasting.

### Managers

**AuctionManager** (managers/auction.rs):
- Creates JIT and AOT auctions with appropriate rules
- Validates bids against minimum requirements
- Resolves auctions when conditions are met
- Returns losers for refund processing

**GameManager** (managers/game.rs):
- Tracks player statistics per session
- Calculates progression (XP, levels, achievements)
- Generates leaderboards across multiple dimensions
- Processes win/loss outcomes with streak tracking

**SessionManager** (managers/session.rs):
- Creates unique session identifiers
- Validates session expiration
- Extends session lifetime on activity
- Cleans up expired sessions periodically

### Models

**Auction Models** (models/auction.rs):
- `JitAuction`: sealed-bid auction for next slot
- `AotAuction`: English auction for future slot
- Bid validation logic per auction type

**Transaction Models** (models/transaction.rs):
- `TransactionStatus` enum: Pending, `Included`, `Failed`, `AuctionWon`
- `InclusionType` enum: `Jit` or `Aot` with reserved slot
- State transition methods (`mark_included`, `mark_failed`, etc.)

**Slot Models** (models/slot.rs):
- `SlotState` enum representing lifecycle
- Slot structure with compute unit tracking
- State transition methods (`reserve`, `fill`)

**Marketplace Models** (models/marketplace.rs):
- `SlotMarketplace` maintaining rolling window
- Slot initialization and advancement logic
- Base fee calculation

### Routes

HTTP handlers that:
1. Extract session from cookies or query parameters
2. Validate request parameters
3. Coordinate manager operations
4. Return standardized API responses
5. Handle errors with appropriate status codes

All routes documented via utoipa macros for OpenAPI generation.

### Services

**Session Service** (services/session.rs):
- Extracts session ID from cookie or query parameter
- Validates session with `SessionManager`
- Returns `Result<String, StatusCode>` for downstream use

**Transaction Service** (services/transaction.rs):
- Updates transaction status for winners
- Processes refunds for losers
- Coordinates with `GameManager` for player stats
- Handles both JIT and AOT resolution patterns

## Concurrency Model

### Read-Write Lock Strategy

State components use `Arc<RwLock<T>>`:
- Multiple concurrent readers allowed
- Exclusive write access when mutating
- Short critical sections to minimize contention

### Lock Ordering

To prevent deadlocks, locks acquired in consistent order:
1. marketplace
2. auctions
3. transactions
4. game

Services release locks between manager calls to prevent holding multiple locks simultaneously.

### Event Broadcasting

`tokio::sync::broadcast` channel for SSE:
- Non-blocking send (best effort delivery)
- Each client gets independent receiver
- Channel capacity 10,000 events
- Dropped events acceptable for real-time updates

## Data Flow Examples

### JIT Bid Submission
```
1.  POST /transactions/jit
2.  Extract session from cookie
3.  Lock game state
4.  Validate balance
5.  Deduct balance
6.  Track bid in player stats
7.  Release game lock
8.  Lock auction manager
9.  Submit bid to JIT auction
10. Release auction lock
11. Lock marketplace
12. Update slot state
13. Release marketplace lock
14. Create transaction record
15. Lock transaction store
16. Add transaction
17. Release transaction lock
18. Broadcast JitBidSubmitted event
19. Return success response
```

### AOT Auction Resolution
```
1. Background task detects slot reached
2. Lock auction manager
3. Get ready AOT auctions
4. For each auction:
   a. Resolve to find winner
   b. Collect losers with bid amounts
5. Remove resolved auctions
6. Release auction lock
7. Broadcast AotAuctionResolved events
8. For each auction:
   a. Lock marketplace
   b. Reserve slot for winner
   c. Release marketplace lock
   d. Update winner transactions
   e. Lock game state
   f. Mark winner auction resolved
   g. Process winner (increment wins, XP)
   h. For each loser:
      i. Refund bid amount
      ii. Mark auction resolved
      iii. Process loss (reset streak)
   i. Release game lock
```

## API Response Format

All endpoints return standardized ApiResponse:
```rust
{
  "success": bool,
  "message": string,
  "data": object | null,
  "code": number
}
```

Success responses include relevant data. Failure responses include error message and HTTP status code.

## Real-time Updates

Server-Sent Events stream over /events endpoint:

Event Types:
- `SlotAdvanced`: current slot incremented
- `SlotsUpdated`: slot states changed
- `JitAuctionStarted`: new JIT auction created
- `AotAuctionStarted`: new AOT auction created
- `JitBidSubmitted`: bid placed in JIT auction
- `AotBidSubmitted`: bid placed in AOT auction
- `JitAuctionResolved`: JIT winner determined
- `AotAuctionResolved`: AOT winner determined
- `TransactionUpdated`: transaction status changed
- `MarketplaceStats`: periodic statistics

Frontend subscribes via EventSource API and updates UI reactively.

## Error Handling

Routes return `Result<Response, StatusCode>` with specific codes:
- 200: Success
- 400: Bad Request (invalid parameters)
- 401: Unauthorized (missing/invalid session)
- 402: Payment Required (insufficient balance)
- 404: Not Found (resource doesn't exist)
- 429: Too Many Requests (rate limit exceeded)
- 500: Internal Server Error (unexpected failure)

Manager operations return anyhow::Result for error context.

## Configuration

Environment variables control:
- Server bind address and port
- CORS allowed origins
- Slot duration in milliseconds
- Base fee in SOL
- Slot advancement interval
- AOT auction default duration

Loaded via `dotenvy` from .env file or environment.