use std::net::SocketAddr;
use axum::Router;
use apex_router::computer_use_api; // ensure module is visible

#[tokio::main]
async fn main() {
    // Build the API router mounted under /api/v1/computer-use
    let app: Router = computer_use_api::router();
    // Bind to 127.0.0.1:3001
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Starting Apex Router (Computer Use MVP) at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
