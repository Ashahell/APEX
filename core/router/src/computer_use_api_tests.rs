#[cfg(test)]
mod tests {
    use super::super::router as _; // bring the router into scope
    use axum::{body::Body, http::{Request, StatusCode, Method}};
    use tower::ServiceExt;
    use hyper::body::to_bytes;

    #[tokio::test]
    async fn hands_start_endpoint_exists_and_responds() {
        let app = crate::computer_use_api::router();
        let req = Request::builder()
            .method(Method::POST)
            .uri("/hands/start")
            .header("content-type", "application/json")
            .body(Body::from("{\"name\": \"computer-use\"}"))
            .unwrap();
        let resp = app.clone().oneshot(req).await.expect("hands_start failed");
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
