# API Security Implementation (MVP: Streaming & REST)

- Implement a minimal HMAC-based authentication wrapper for REST endpoints.
- Add a simple timestamp freshness check (5-minute replay protection).
- Introduce a minimal audit log integration to record API calls.
- Apply cookie-free, token-less gating for MVP; future: token-based API keys.
- Add basic rate limiting (per IP) stubs for MVP.
