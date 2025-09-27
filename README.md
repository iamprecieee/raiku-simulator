# Raiku Simulator Developer Guide

## Overview

The `raiku-simulator` is a Rust-based backend application designed to simulate a marketplace for transaction slot auctions. It supports two types of auctions:
- **Just-in-Time (JIT) Auctions**: For transactions that need to be included in the very next available slot.
- **Ahead-of-Time (AoT) Auctions**: For transactions that are scheduled for future slots, allowing for a bidding period.

The simulator provides a RESTful API for interacting with the marketplace, managing user sessions, submitting bids, and querying the state of slots and transactions. It also includes a Server-Sent Events (SSE) endpoint for real-time updates on marketplace activities.

## Architecture

The `raiku-simulator` is built using the `axum` web framework and `tokio` for asynchronous runtime. It employs a layered architecture with a central `AppState` managing various concurrent components using `Arc<RwLock<T>>` for thread-safe access to shared mutable data.

**Key Architectural Principles:**
- **Asynchronous Processing**: Leverages `tokio` for efficient handling of concurrent operations, including API requests, scheduled tasks, and event broadcasting.
- **Shared State Management**: `AppState` acts as the single source of truth, providing controlled access to core components like the marketplace, auction manager, and transaction store.
- **Event-Driven Communication**: Uses an `EventBroadcaster` to disseminate significant application events, enabling real-time updates to connected clients via SSE.
- **Modular Design**: The codebase is organized into distinct modules, each responsible for a specific domain (e.g., `api`, `state`, `auction`, `session`, `slot`, `transaction`, `events`, `rate_limiter`, `config`).

## Module Breakdown

### `main.rs`
The application's entry point. It handles:
- Initialization of logging.
- Loading application configuration from environment variables.
- Setting up the `AppState` and `RateLimiter`.
- Spawning background `tokio` tasks for:
    - Periodically advancing the slot and resolving JIT/AoT auctions.
    - Cleaning up expired user sessions.
- Configuring and starting the `axum` HTTP server.

### `lib.rs`
The library root that re-exports all public modules and defines common enums:
- `InclusionType`: Differentiates between JIT and AoT transaction inclusion.
- `TransactionType`: Distinguishes between JIT and AoT transactions.

### `api.rs`
Defines the REST API endpoints and their handlers.
- **`AppContext`**: A struct holding `AppState`, `Config`, and `RateLimiter`, passed as application state to API handlers.
- **Routes**: Configures all API routes for sessions, marketplace status, slots, auctions, transactions, and health checks.
- **Middleware**: Integrates rate limiting and CORS policies.

### `state.rs`
Manages the global application state and provides methods for interacting with core components.
- **`AppState`**: Contains `Arc<RwLock<T>>` wrappers for:
    - `SlotMarketplace`: Manages the state of all transaction slots.
    - `AuctionManager`: Handles JIT and AoT auction logic.
    - `HashMap<String, Transaction>`: Stores all transactions by ID.
    - `HashMap<String, Vec<String>>`: Maps session IDs to their transaction IDs.
    - `SessionManager`: Manages user sessions.
    - `EventBroadcaster`: Publishes application events.
- **Core Logic**: Provides high-level methods for adding/updating transactions, advancing slots, retrieving marketplace statistics, starting auctions, and submitting bids.

### `auction.rs`
Implements the logic for Just-in-Time (JIT) and Ahead-of-Time (AoT) auctions.
- **`Bid`**: Represents a bid placed by a user for a slot.
- **`JiTAuction`**: Handles bids for the very next slot, with a simple highest-bid-wins mechanism.
- **`AoTAuction`**: Manages auctions for future slots, including a bidding period and resolution logic based on time or slot advancement.
- **`AuctionManager`**: Stores and manages active JIT and AoT auctions, providing methods to start, submit bids, and resolve them.

### `session.rs`
Manages user sessions for API interaction.
- **`Session`**: Represents an individual user session with an ID, creation time, last active time, and expiration time.
- **`SessionManager`**: Provides functionality to create new sessions, retrieve and validate existing sessions (extending their expiry on activity), and clean up expired sessions.

### `events.rs`
Provides a publish-subscribe mechanism for application events.
- **`AppEvent`**: An enum enumerating all possible events within the simulator (e.g., slot advanced, auction started, bid submitted, transaction updated, marketplace stats). These events are serializable for SSE.
- **`EventBroadcaster`**: Allows components to broadcast events and provides `Receiver`s for subscribers (like the `/events` SSE endpoint).

### `rate_limiter.rs`
Implements API rate limiting to prevent abuse.
- **`RateLimiter`**: Uses a `DashMap` of `TokenBucket`s to track request counts per client IP within a sliding window.
- **`TokenBucket`**: Stores rate limiting state for a single client, including tokens, last refill time, and request count.
- **`rate_limit_middleware`**: An `axum` middleware function that intercepts incoming requests, checks against the rate limit, and blocks requests if the limit is exceeded.

### `slot.rs`
Manages the concept of "slots" – discrete blockspace units in which transactions can be included.
- **`SlotState`**: An enum representing the current state of a slot (e.g., `Available`, `JiTAuction`, `AoTAuction`, `Reserved`, `Filled`, `Expired`).
- **`Slot`**: Represents a single slot, including its number, state, estimated time, base fee, and compute unit availability/usage.
- **`SlotMarketplace`**: Holds and manages a collection of `Slot`s, tracks the `current_slot`, and handles advancing the slot. It initializes a rolling window of upcoming slots.

### `transaction.rs`
Defines the structure and lifecycle of a transaction.
- **`TransactionStatus`**: An enum tracking the status of a transaction (e.g., `Pending`, `Included`, `Failed`, `AuctionWon`).
- **`Transaction`**: Contains details such as ID, sender, inclusion type, current status, compute units, priority fee (bid amount), and timestamps. Provides methods to create JIT/AoT transactions and update their status.

### `config.rs`
Handles application configuration, primarily loading settings from environment variables.
- **`Config`**: The main configuration struct, composed of:
    - `ServerConfig`: HTTP server settings (host, port, CORS origins).
    - `MarketplaceConfig`: Marketplace-specific settings (slot duration, base fee, slot advance interval).
    - `AuctionConfig`: Auction-specific settings (AoT default duration).
- **`from_env()`**: A constructor that reads environment variables, applying default values if not present, and parses them into the configuration structs.

## 4. API Endpoints

All API endpoints are prefixed with the server host and port (e.g., `http://localhost:8080`).

### Session Management

-   `POST /sessions`
    -   **Description**: Creates a new session or validates an existing one.
    -   **Request Body**:
        ```json
        {
            "session_id": "optional_existing_session_id"
        }
        ```
    -   **Response**:
        ```json
        {
            "session_id": "new_or_validated_session_id",
            "status": "created" | "validated",
            "created_at": "timestamp",
            "expires_at": "timestamp"
        }
        ```

### Event Stream

-   `GET /events`
    -   **Description**: Establishes a Server-Sent Events (SSE) connection to receive real-time application events.
    -   **Events Streamed**: `SlotAdvanced`, `SlotsUpdated`, `JitAuctionStarted`, `AotAuctionStarted`, `JitBidSubmitted`, `AotBidSubmitted`, `JitAuctionResolved`, `AotAuctionResolved`, `TransactionUpdated`, `MarketplaceStats`.

### Marketplace Information

-   `GET /marketplace/status`
    -   **Description**: Returns the current status of the marketplace.
    -   **Response**:
        ```json
        {
            "current_slot": 123,
            "stats": {
                "current_slot": 123,
                "total_slots": 100,
                "active_jit_auctions": 1,
                "active_aot_auctions": 5,
                "total_transactions": 50
            },
            "slot_time_ms": 400,
            "base_fee_sol": 0.001
        }
        ```

-   `GET /marketplace/slots`
    -   **Description**: Lists upcoming slots. Requires `session_id` query parameter.
    -   **Query Parameters**:
        -   `session_id` (string, required): The ID of the requesting session.
    -   **Response**:
        ```json
        {
            "session_id": "your_session_id",
            "current_slot": 123,
            "slots": [
                {
                    "slot_number": 123,
                    "state": "Available" | "JiTAuction" | "AoTAuction" | "Reserved" | "Filled" | "Expired",
                    "estimated_time": "timestamp",
                    "base_fee": 0.001,
                    "compute_units_available": 48000000,
                    "compute_units_used": 0
                }
            ]
        }
        ```

-   `GET /marketplace/slots/{slot_number}`
    -   **Description**: Retrieves details for a specific slot.
    -   **Path Parameters**:
        -   `slot_number` (u64, required): The number of the slot to retrieve.
    -   **Response**:
        ```json
        {
            "slot_number": 123,
            "state": "Available",
            "estimated_time": "timestamp",
            "base_fee": 0.001,
            "compute_units_available": 48000000,
            "compute_units_used": 0
        }
        ```

### Auction Information

-   `GET /auctions/jit`
    -   **Description**: Lists all active Just-in-Time (JIT) auctions.
    -   **Response**:
        ```json
        {
            "auctions": [
                {
                    "slot_number": 124,
                    "min_bid": 0.00105,
                    "current_winner": ["session_abc", 0.0011],
                    "created_at": "timestamp"
                }
            ],
            "count": 1
        }
        ```

-   `GET /auctions/aot`
    -   **Description**: Lists all active Ahead-of-Time (AoT) auctions.
    -   **Response**:
        ```json
        {
            "auctions": [
                {
                    "slot_number": 125,
                    "min_bid": 0.001,
                    "highest_bid": 0.0012,
                    "bids_count": 3,
                    "ends_at": "timestamp",
                    "has_ended": false
                }
            ],
            "count": 1
        }
        ```

### Transaction Submission

-   `POST /transactions/jit`
    -   **Description**: Submits a Just-in-Time (JIT) transaction bid for the next available slot.
    -   **Request Body**:
        ```json
        {
            "session_id": "your_session_id",
            "bid_amount": 0.0015,
            "compute_units": 200000,
            "data": "your_transaction_data"
        }
        ```
    -   **Response**:
        ```json
        {
            "transaction_id": "uuid_of_transaction",
            "slot_number": 124,
            "bid_amount": 0.0015,
            "status": "auction_pending",
            "message": "JIT bid submitted for next available slot"
        }
        ```

-   `POST /transactions/aot`
    -   **Description**: Submits an Ahead-of-Time (AoT) transaction bid for a specific future slot.
    -   **Request Body**:
        ```json
        {
            "session_id": "your_session_id",
            "slot_number": 125,
            "bid_amount": 0.0012,
            "compute_units": 250000,
            "data": "your_transaction_data"
        }
        ```
    -   **Response**:
        ```json
        {
            "transaction_id": "uuid_of_transaction",
            "slot_number": 125,
            "bid_amount": 0.0012,
            "status": "auction_pending",
            "message": "AOT bid submitted for future slot"
        }
        ```

### Transaction Information

-   `GET /transactions`
    -   **Description**: Lists transactions. Can filter by `session_id` or show all if `show_all=true`. Supports pagination.
    -   **Query Parameters**:
        -   `session_id` (string, optional): Filter transactions by this session ID. Required if `show_all` is not `true`.
        -   `page` (u32, optional, default: 1): The page number for pagination.
        -   `limit` (u32, optional, default: 20, max: 100): The number of transactions per page.
        -   `show_all` (bool, optional, default: false): If `true`, lists all transactions across all sessions.
    -   **Response (filtered by session_id)**:
        ```json
        {
            "session_id": "your_session_id",
            "transactions": [...],
            "pagination": {
                "current_page": 1,
                "total_pages": 5,
                "page_size": 20,
                "total_count": 90,
                "has_next": true,
                "has_prev": false
            },
            "showing": "session_only"
        }
        ```
    -   **Response (show_all=true)**:
        ```json
        {
            "transactions": [...],
            "pagination": {
                "current_page": 1,
                "total_pages": 5,
                "page_size": 20,
                "total_count": 90,
                "has_next": true,
                "has_prev": false
            },
            "showing": "all"
        }
        ```

-   `GET /transactions/all`
    -   **Description**: Lists all transactions across all sessions. Supports pagination.
    -   **Query Parameters**:
        -   `page` (u32, optional, default: 1): The page number for pagination.
        -   `limit` (u32, optional, default: 20, max: 100): The number of transactions per page.
    -   **Response**: Same as `GET /transactions?show_all=true`.

-   `GET /transactions/{transaction_id}`
    -   **Description**: Retrieves a specific transaction by its ID.
    -   **Path Parameters**:
        -   `transaction_id` (string, required): The ID of the transaction to retrieve.
    -   **Response**:
        ```json
        {
            "id": "uuid_of_transaction",
            "sender": "session_abc",
            "inclusion_type": "JiT",
            "status": {
                "Included": {
                    "slot": 124,
                    "execution_time": "timestamp"
                }
            },
            "compute_units": 200000,
            "priority_fee": 0.0015,
            "data": "your_transaction_data",
            "created_at": "timestamp",
            "included_at": "timestamp"
        }
        ```

### Health Check

-   `GET /health`
    -   **Description**: Simple endpoint to check the health of the API.
    -   **Response**:
        ```json
        {
            "status": "healthy",
            "timestamp": "timestamp"
        }
        ```

## 5. Setup and Running

### Prerequisites

-   Rust (latest stable version recommended)
-   Cargo (Rust's package manager, installed with Rust)
-   Docker (optional, for containerized deployment)

### Local Development

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-repo/raiku-simulator.git
    cd raiku-simulator
    ```

2.  **Environment Variables**:
    Create a `.env` file in the project root with the following (or set them directly in your environment):
    ```
    SERVER_HOST=0.0.0.0
    SERVER_PORT=8080
    CORS_ORIGINS=http://localhost:3000,http://127.0.0.1:3000
    SLOT_TIME_MS=400
    BASE_FEE_SOL=0.001
    ADVANCE_SLOT_INTERVAL_MS=400
    AOT_DURATION_SEC=35
    ```

3.  **Run the application**:
    ```bash
    cargo run
    ```
    The application will start and be accessible at `http://0.0.0.0:8080` (or your configured host/port).

### Docker Deployment

1.  **Build the Docker image**:
    ```bash
    docker build -t raiku-simulator .
    ```

2.  **Run the Docker container**:
    ```bash
    docker run -p 8000:8000 -e RUST_LOG=info -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8000 raiku-simulator
    ```
    The application will be accessible at `http://localhost:8000`.

## 6. Project Structure

```
.
├── Cargo.toml                # Rust project manifest and dependencies
├── Dockerfile                # Docker build instructions
├── src/
│   ├── main.rs               # Main application entry point
│   ├── lib.rs                # Library root, module re-exports, common enums
│   ├── api.rs                # Defines API routes and handlers
│   ├── auction.rs            # JIT and AoT auction logic and management
│   ├── config.rs             # Environment variable based configuration loading
│   ├── events.rs             # Event broadcasting and subscription system
│   ├── rate_limiter.rs       # API rate limiting middleware
│   ├── session.rs            # User session management
│   ├── slot.rs               # Slot definition and marketplace management
│   ├── state.rs              # Global application state and shared data access
│   └── transaction.rs        # Transaction definition and status management
└── raiku-frontend/           # (Separate frontend application, not part of this guide)
    └── ...
```