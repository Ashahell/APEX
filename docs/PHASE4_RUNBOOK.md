Phase 4 Runbook: Backups, DR, Runbooks
====================================

Overview
- This runbook documents Phase 4: backups, disaster recovery (DR), and runbooks for APEX production.
- Objective is to ensure data protection, recoverability, and operational playbooks for incident response.

Scope
- Production data: PostgreSQL DB (apex) and memory store if applicable
- Configuration and secrets: deployment configs and TLS certs (with sensitive values redacted)
- Runbooks: backup, restore, DR tests, incident response, and handover artifacts

Prerequisites
- Access to docker and docker-compose, access to secrets store, and a writable backups directory
- Docker secrets configured (db_password)
- Correct network access to backups location (local/remote)

Backups
- Schedule: daily incremental; weekly full backups
- Retention: 6 weeks
- Backup artifacts: backups/ directory, with subfolders per date
- Procedures:
  - Run ./scripts/backup_all.sh <target-backup-dir>
  - Verify backups exist: ls -l <backup-dir>
  - Log backups in a central index (backups/backup_index.md)

Disaster Recovery (DR)
- DR test plan: quarterly DR test in a staging environment
- Steps:
  - Bring up prod stack in staging using prod TLS/keys as applicable
  - Execute backup restore sequence on a test DB and memory
  - Validate data integrity (row counts, checksums)
  - Validate failover/rollback paths
  - Capture runbook execution results and update DR_runbook logs

Runbooks
- Phase 4 DR Runbooks are stored in docs/PHASE4_DR_TEST_PLAN.md and PHASE4_RUNBOOK.md
- Incident response: runbooks for containment, eradication, recovery, and lessons learned

Validation & Sign-off
- Runbooks reviewed and signed off by Security and Ops
- DR test results recorded and archived with parity artifacts
