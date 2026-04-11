Phase 3 Smoke Runbook
=====================
- Objective: Validate prod-stack health via smoke tests against core endpoints.
- Preconditions: prod stack up via docker-compose.prod.yml, images cached or built.
- Steps:
  1) Start stack: docker-compose -f docker-compose.prod.yml up -d --build
  2) Wait for services to initialize (approx 40s)
  3) Run smoke: bash scripts/prod_smoke.sh
  4) If any endpoint fails, collect container logs and surface via CI artifact
  5) Tear down: docker-compose -f docker-compose.prod.yml down -v
- Output: phase3-smoke_report.txt (or integrated artifact)
