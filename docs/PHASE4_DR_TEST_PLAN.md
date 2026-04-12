# Phase 4 DR Test Plan
Overview
- Purpose: Validate DR readiness for APEX production by testing backup restoration, failover, and recovery processes.

Scope
- PostgreSQL DB backup/restore, memory store restoration, and deployment bring-up in a staging environment.

Test Scenarios
- DR-01: Restore DB from latest backup and verify data integrity
- DR-02: Restore full environment in staging from backup artifacts
- DR-03: Simulate partial failover (router or UI) with quick rollback
- DR-04: End-to-end restore and verify service health

Prerequisites
- Backups exist and accessible
- Staging environment configured to resemble production
- TLS/secret handling tested in staging

Test Steps (for each DR scenario)
- Step 1: Prepare environment (clean state)
- Step 2: Restore data from backup artifacts to staging DB
- Step 3: Bring up services and verify health checks pass
- Step 4: Validate data integrity (row counts, checksums)
- Step 5: Execute failover/rollback if applicable
- Step 6: Document results and capture logs

Acceptance Criteria
- All DR scenarios pass within defined RTO/RPO
- Data integrity verified
- Runbooks updated with outcomes and improvements
