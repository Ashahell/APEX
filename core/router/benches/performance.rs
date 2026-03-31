//! Performance Benchmarks for APEX Router
//!
//! Benchmarks for critical paths: streaming, telemetry, memory, MCP

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_injection_classifier(c: &mut Criterion) {
    let inputs = vec![
        ("safe_query", "What is the weather like today?"),
        ("sql_injection", "'; DROP TABLE users; --"),
        ("command_injection", "echo hello; rm -rf /"),
        ("path_traversal", "../../../etc/passwd"),
        ("xss", "<script>alert('xss')</script>"),
    ];

    let mut group = c.benchmark_group("injection_classifier");
    for (name, input) in inputs {
        group.bench_function(name, |b| {
            b.iter(|| {
                apex_router::security::injection_classifier::InjectionClassifier::analyze(
                    black_box(input),
                )
            })
        });
    }
    group.finish();
}

fn benchmark_telemetry_latency_tracker(c: &mut Criterion) {
    let mut group = c.benchmark_group("telemetry_latency");

    group.bench_function("record_100", |b| {
        b.iter(|| {
            let tracker = apex_router::metrics::EndpointLatencyTracker::new(1000);
            for i in 0..100u64 {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(tracker.record(black_box(i)));
            }
        })
    });

    group.bench_function("get_stats_100_entries", |b| {
        let tracker = apex_router::metrics::EndpointLatencyTracker::new(1000);
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..100u64 {
            rt.block_on(tracker.record(i));
        }
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(tracker.get_stats())
        })
    });

    group.finish();
}

fn benchmark_replay_protection(c: &mut Criterion) {
    let mut group = c.benchmark_group("replay_protection");

    group.bench_function("record_and_check", |b| {
        b.iter(|| {
            apex_router::security::replay_protection::reset();
            for i in 0..100 {
                let sig = format!("bench-sig-{}", i);
                apex_router::security::replay_protection::record_and_check(black_box(&sig));
            }
        })
    });

    group.finish();
}

fn benchmark_memory_validation(c: &mut Criterion) {
    use serde_json::json;

    let mut group = c.benchmark_group("memory_validation");

    group.bench_function("validate_small_payload", |b| {
        b.iter(|| {
            apex_router::mcp::validation::sanitize_tool_arguments(black_box(&json!({
                "name": "test",
                "value": 123
            })))
        })
    });

    group.bench_function("validate_nested_payload", |b| {
        b.iter(|| {
            apex_router::mcp::validation::sanitize_tool_arguments(black_box(&json!({
                "user": {
                    "name": "test",
                    "settings": {
                        "theme": "dark",
                        "notifications": true
                    }
                }
            })))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_injection_classifier,
    benchmark_telemetry_latency_tracker,
    benchmark_replay_protection,
    benchmark_memory_validation,
);
criterion_main!(benches);
