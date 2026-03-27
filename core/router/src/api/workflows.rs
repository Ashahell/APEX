#![allow(unused_imports)]

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use ulid::Ulid;

use apex_memory::{CreateWorkflow, UpdateWorkflow};

use super::{
    AppState, CreateWorkflowRequest, ListWorkflowsQuery, UpdateWorkflowRequest,
    WorkflowExecutionResponse, WorkflowResponse,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/workflows",
            get(list_workflows).post(create_workflow),
        )
        .route(
            "/api/v1/workflows/filter-options",
            get(get_workflow_filter_options),
        )
        .route(
            "/api/v1/workflows/:id",
            get(get_workflow)
                .put(update_workflow)
                .delete(delete_workflow),
        )
        .route(
            "/api/v1/workflows/:id/executions",
            get(get_workflow_executions),
        )
}

async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<ListWorkflowsQuery>,
) -> Result<Json<Vec<WorkflowResponse>>, String> {
    let repo = &state.workflow_repo;
    let workflows = if query.active_only.unwrap_or(false) {
        repo.find_active().await.map_err(|e| e.to_string())?
    } else if let Some(ref cat) = query.category {
        repo.find_by_category(cat)
            .await
            .map_err(|e| e.to_string())?
    } else {
        repo.find_all().await.map_err(|e| e.to_string())?
    };
    Ok(Json(
        workflows.into_iter().map(WorkflowResponse::from).collect(),
    ))
}

async fn get_workflow_filter_options(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = &state.workflow_repo;
    let categories = repo.get_categories().await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "categories": categories })))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let workflow = repo
        .find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let id = Ulid::new().to_string();
    let create = CreateWorkflow {
        name: req.name,
        description: req.description,
        definition: req.definition,
        category: req.category,
    };
    repo.create(&id, &create).await.map_err(|e| e.to_string())?;
    let workflow = repo
        .find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found after creation".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, String> {
    let repo = &state.workflow_repo;
    let update = UpdateWorkflow {
        name: req.name,
        description: req.description,
        definition: req.definition,
        category: req.category,
        is_active: req.is_active,
    };
    repo.update(&id, &update).await.map_err(|e| e.to_string())?;
    let workflow = repo
        .find_by_id(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found after update".to_string())?;
    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = &state.workflow_repo;
    repo.delete(&id).await.map_err(|e| e.to_string())?;
    Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
}

async fn get_workflow_executions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<WorkflowExecutionResponse>>, String> {
    let repo = &state.workflow_repo;
    let executions = repo
        .get_executions(&id, 50)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(
        executions
            .into_iter()
            .map(WorkflowExecutionResponse::from)
            .collect(),
    ))
}
