Title: API Surface for Computer Use (Appendix A mappings)
Date: 2026-03-25
Status: Draft

Overview
- This document maps the Appendix A style endpoint definitions from docs/COMPUTER_USE_IMPLEMENTATION.md into a concrete API surface that can be implemented in core/router and gateway layers.

Endpoints
- POST /api/v1/computer-use/execute
  - Description: Start a computer-use task
  - Payload (example): { task: "Book a flight", max_steps: 20, max_cost_usd: 5.0, stream: true }
  - Response: { success: boolean, steps: number, cost: number, final_state?: object, error?: string }

- GET /api/v1/computer-use/status/:id
  - Description: Get status of a running/rerun computer-use task
  - Response: { id: string, status: string, progress: number, cost: number, result?: object }

- POST /api/v1/computer-use/cancel/:id
  - Description: Cancel a running task
  - Response: { success: boolean, message?: string }

- GET /api/v1/computer-use/stream/:id
  - Description: Server-Sent Events / WebSocket stream for live screenshots and progress
  - Response: Stream of events containing screenshot frames or metadata

- GET /api/v1/computer-use/screenshots/:id
  - Description: Retrieve screenshot history for a task
  - Response: Array of CapturedScreenshot-like objects

Notes
- The actual implementation may deviate in naming or shape as the codebase evolves; this document captures the intended surface for MVP compatibility.
