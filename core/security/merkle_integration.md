# Merkle Audit Trail Integration (OpenFang -> APEX) - MVP sketch

- Concept: adopt a Merkle-style audit trail for action histories. Each action is hashed and chained to the previous, enabling tamper evaluation.
- MVP: a simple in-memory Merkle tree-like structure with append-only log entries and a finalize hash.
- Will integrate with the MVP orchestrator to record actions and produce a verifiable root hash.
