# API Security Tests (MVP)

- Verify HMAC signature validation and timestamp freshness
- Validate a valid signed request succeeds for the MVP surface
- Validate an expired timestamp or invalid signature fails
- Ensure audit logs are produced for API calls (mocked)
