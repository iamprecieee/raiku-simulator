# API Integration Guide

## Authentication

All endpoints requiring user identity use cookie-based sessions. Create a session first, then include the cookie in subsequent requests.

### Create Session
```bash
curl -X POST http://localhost:8080/sessions \
  -H "Content-Type: application/json" \
  -c cookies.txt
```

Response:
```json
{
  "success": true,
  "message": "Session created.", // or validated
  "data": {
    "session_id": "a1b2c3d4-e5f6-4789-0abc-def123456789",
    "status": "created",
    "created_at": "2025-01-15T10:30:00Z",
    "expires_at": "2025-01-16T10:30:00Z"
  },
  "code": 200
}
```

The `Set-Cookie` header contains `raiku_session` token. Include in future requests via `-b cookies.txt` flag or `Cookie` header.

## Marketplace Endpoints

### Get Marketplace Status
```bash
curl http://localhost:8080/marketplace/status
```

Returns current slot, active auction counts, and configuration.

### List Available Slots
```bash
curl http://localhost:8080/marketplace/slots
```

Returns next 50 slots with states, times, and fees.

### Get Specific Slot
```bash
curl http://localhost:8080/marketplace/slots/125
```

Returns detailed information for slot 125.

## Auction Participation

### Submit JIT Bid

Bid for immediate inclusion in next available slot.
```bash
curl -X POST http://localhost:8080/transactions/jit \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "bid_amount": 0.005,
    "compute_units": 200000,
    "data": "urgent_transaction"
  }'
```

Parameters:
- `bid_amount`: SOL amount willing to pay (must exceed minimum)
- `compute_units`: compute units required (max 48,000,000)
- `data`: transaction payload string

Response includes `transaction_id` for tracking and `slot_number` where bid was placed.

### Submit AOT Bid

Reserve specific future slot (must be 35+ slots ahead).
```bash
curl -X POST http://localhost:8080/transactions/aot \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "slot_number": 150,
    "bid_amount": 0.003,
    "compute_units": 250000,
    "data": "scheduled_settlement"
  }'
```

Parameters:
- `slot_number`: target slot for reservation
- `bid_amount`: SOL amount willing to pay
- `compute_units`: compute units required
- `data`: transaction payload string

Response includes auction end time. Can bid multiple times before auction closes.

## Transaction Tracking

### List Transactions

View your transaction history with pagination.
```bash
curl "http://localhost:8080/transactions?page=1&limit=20" \
  -b cookies.txt
```

Query Parameters:
- `page`: page number (default 1)
- `limit`: items per page (default 20, max 100)
- `show_all`: true to see all transactions (default false)

### Get Transaction Details
```bash
curl http://localhost:8080/transactions/abc123def456 \
  -b cookies.txt
```

Returns full transaction object with current status.

## Auction Information

### Active JIT Auctions
```bash
curl http://localhost:8080/auctions/jit
```

Returns list of active JIT auctions with current highest bidders and minimum bids.

### Active AOT Auctions
```bash
curl http://localhost:8080/auctions/aot
```

Returns list of active AOT auctions with bid counts, highest bids, and end times.

## Player Statistics

### Get Player Stats
```bash
curl http://localhost:8080/game/player_stats \
  -b cookies.txt
```

Returns balance, wins, level, streak, achievements, and participation data.

### Get Leaderboard
```bash
curl http://localhost:8080/game/leaderboard
```

Returns top 10 players across three categories: total wins, highest balance, best win rate.

## Real-time Updates

### Subscribe to Events
```javascript
const eventSource = new EventSource('http://localhost:8080/events');

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event received:', data);
};

eventSource.onerror = () => {
  console.log('Connection lost, reconnecting...');
};
```

Event types and their data structures documented in ARCHITECTURE.md.

## Error Responses

All errors return:
```json
{
  "success": false,
  "message": "error description",
  "code": 400
}
```

Common error codes:
- 400: Invalid request parameters
- 401: Missing or invalid session
- 402: Insufficient balance
- 404: Resource not found
- 429: Rate limit exceeded

## Rate Limiting

API enforces 6000 requests per minute per IP address. Exceeded requests return 429 status.


## OpenAPI Documentation

Interactive API documentation available at:
```
http://localhost:8080/swagger-ui
```