# Phase 0 Tracker Import Instructions

Overview
- This document provides step-by-step instructions to import Phase 0 tickets into your project tracker (e.g., Jira, YouTrack, GitHub Projects).
- It also shows how to map JSON fields to tracker fields and how to attach evidence artifacts.

Prerequisites
- Access to your tracker's REST API or UI.
- API tokens or credentials configured in your environment.
- Phase 0 tickets JSON payload available at docs/phase0_tickets.json.

Importer workflow (example for generic tracker)
1) Prepare payload
- Use docs/phase0_tickets.json as the source payload. Ensure IDs are prefixed with NDEV-P0-XX.
2) Create issues in tracker
- For each ticket, create an issue with fields:
  - Summary: ticket.title
  - Description: ticket.description
  - Type: ticket.type
  - Priority: ticket.priority
  - Assignee: ticket.assignee (default: TBD)
  - Reporter: current user
  - Labels: parity, phase0
  - Custom fields: estimate (ticket.estimate)
3) Attachments
- If available, attach the JSON payload and crosswalk templates as artifacts to the respective tickets.
4) Sign-off
- After import, run a Phase 0 kickoff to align owners and milestones.

Examples
- Import payload via REST (pseudo-commands):
  curl -X POST https://your-tracker.example/api/issues \
    -H "Authorization: Bearer <token>" \
    -H "Content-Type: application/json" \
    -d @docs/phase0_tickets.json

- Update: set assignee and estimate after import as needed.

Artifacts
- Phase 0 tickets JSON: docs/phase0_tickets.json
- Phase 0 governance: docs/phase0_gating.md
