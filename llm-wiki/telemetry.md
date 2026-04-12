# Telemetry

## Overview

APEX implements comprehensive telemetry for monitoring system health, performance, and user behavior.

## Metrics

### System Metrics
- Per-endpoint latency (p50, p95, p99)
- Error rates by endpoint and error type
- Request throughput (RPM/RPS)
- SLO tracking (availability, latency)

### Custom Metrics
- Task execution time and cost
- Skill invocation counts
- Memory usage per store
- Session metrics

## Collection

### Middleware
- `telemetry_middleware.rs` - Per-request metrics
- `metrics.rs` - Aggregated counters/gauges
- `circuit_breaker.rs` - Failure tracking

### Storage
- In-memory metrics with periodic flush
- SQLite backend for historical data
- Prometheus-compatible endpoints

## API Endpoints
- `/api/v1/metrics` - Current metrics
- `/api/v1/system/health` - Health status
- `/api/v1/system/cache` - Cache statistics

## Dashboards
- Real-time monitoring in UI
- Per-endpoint latency graphs
- Error rate alerts
- Session health indicators

## Alerts
- High error rates (configurable threshold)
- Latency p99 exceeding SLA
- Resource exhaustion warnings