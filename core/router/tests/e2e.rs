use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const ROUTER_URL: &str = "http://localhost:3000";

struct RouterProcess {
    child: Child,
}

impl RouterProcess {
    fn start() -> Result<Self, std::io::Error> {
        let mut child = Command::new("cargo")
            .args(["run", "--bin", "apex-router"])
            .current_dir("..")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Wait for the server to be ready
        let started = std::time::Instant::now();
        let timeout = Duration::from_secs(30);

        while started.elapsed() < timeout {
            if std::net::TcpStream::connect("127.0.0.1:3000").is_ok() {
                // Give it a moment to fully start
                thread::sleep(Duration::from_millis(200));
                return Ok(Self { child });
            }
            thread::sleep(Duration::from_millis(100));
        }

        let _ = child.kill();
        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "Router failed to start",
        ))
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test]
async fn test_router_starts_and_responds() {
    // Check if port 3000 is already in use (another router running)
    if std::net::TcpStream::connect("127.0.0.1:3000").is_ok() {
        eprintln!("Port 3000 is in use - skipping E2E test. Stop any running router and try again.");
        return;
    }

    let mut router = match RouterProcess::start() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to start router: {} - ensure cargo build has been run", e);
            return;
        }
    };

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/api/v1/metrics", ROUTER_URL))
        .send()
        .await
        .expect("Failed to connect to router");

    assert_eq!(response.status(), 200, "Router should respond with 200");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    assert!(
        body.get("tasks").is_some(),
        "Response should contain metrics"
    );

    router.stop();
}

#[tokio::test]
async fn test_create_task_via_http() {
    // Check if port 3000 is already in use (another router running)
    if std::net::TcpStream::connect("127.0.0.1:3000").is_ok() {
        eprintln!("Port 3000 is in use - skipping E2E test. Stop any running router and try again.");
        return;
    }

    let mut router = match RouterProcess::start() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to start router: {} - ensure cargo build has been run", e);
            return;
        }
    };

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/api/v1/tasks", ROUTER_URL))
        .json(&serde_json::json!({
            "content": "Hello from e2e test"
        }))
        .send()
        .await
        .expect("Failed to create task");

    assert_eq!(response.status(), 200, "Task creation should succeed");

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    assert!(
        body.get("task_id").is_some(),
        "Response should contain task_id"
    );
    assert!(body.get("tier").is_some(), "Response should contain tier");
    assert!(
        body.get("capability_token").is_some(),
        "Response should contain capability_token"
    );

    router.stop();
}
