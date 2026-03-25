//! Story Engine API
//!
//! REST endpoints for story engine feature.
//!
//! Feature 7: Story Engine

use axum::{
    extract::{State, Path, Query},
    routing::{get, post, delete, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::api::AppState;
use crate::story_engine::{
    DiceRoller, DiceRoll, Story, StoryCharacter, StoryChoice, StoryEngine, StorySetting,
};

/// Create story request
#[derive(Debug, Deserialize)]
pub struct CreateStoryRequest {
    /// Story title
    pub title: String,
    /// Story setting (fantasy, scifi, horror, mystery, western, modern)
    pub setting: String,
    /// Characters in the story
    pub characters: Vec<StoryCharacter>,
}

/// Make choice request
#[derive(Debug, Deserialize)]
pub struct MakeChoiceRequest {
    /// Choice ID to select
    pub choice_id: String,
}

/// Roll dice request
#[derive(Debug, Deserialize)]
pub struct RollDiceRequest {
    /// Dice notation (e.g., "2d6+3")
    pub dice: String,
    /// Optional description
    pub description: Option<String>,
}

/// Advance story request
#[derive(Debug, Deserialize)]
pub struct AdvanceStoryRequest {
    /// Narrative text for this turn
    pub narrative: String,
    /// Available choices for player
    pub choices: Vec<StoryChoice>,
}

/// Story response
#[derive(Debug, Serialize)]
pub struct StoryResponse {
    pub id: String,
    pub title: String,
    pub setting: String,
    pub characters: Vec<StoryCharacter>,
    pub location: String,
    pub inventory: Vec<String>,
    pub turn_count: u32,
    pub available_choices: Vec<StoryChoice>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<&Story> for StoryResponse {
    fn from(story: &Story) -> Self {
        StoryResponse {
            id: story.id.clone(),
            title: story.title.clone(),
            setting: story.setting.as_str().to_string(),
            characters: story.characters.clone(),
            location: story.state.location.clone(),
            inventory: story.state.inventory.clone(),
            turn_count: story.turn_count,
            available_choices: story.state.available_choices.clone(),
            created_at: story.created_at,
            updated_at: story.updated_at,
        }
    }
}

/// Dice roll response
#[derive(Debug, Serialize)]
pub struct DiceRollResponse {
    pub dice: String,
    pub result: u32,
    pub description: String,
}

/// Story list response
#[derive(Debug, Serialize)]
pub struct StoryListResponse {
    pub stories: Vec<StoryResponse>,
    pub count: usize,
}

/// Create story engine router
pub fn create_story_router() -> Router<AppState> {
    Router::new()
        .route("/stories", get(list_stories).post(create_story))
        .route("/stories/:id", get(get_story).delete(end_story))
        .route("/stories/:id/choice", post(make_choice))
        .route("/stories/:id/roll", post(roll_dice))
        .route("/stories/:id/advance", post(advance_story))
        .route("/stories/:id/choices", get(get_choices))
        .route("/stories/:id/location", put(update_location))
        .route("/stories/:id/inventory", get(get_inventory).post(add_inventory))
        .route("/dice/validate", get(validate_dice))
}

/// List all stories
async fn list_stories(State(state): State<AppState>) -> Json<StoryListResponse> {
    let engine = state.story_engine.lock().unwrap();
    let stories: Vec<StoryResponse> = engine.list_stories()
        .iter()
        .map(|s| StoryResponse::from(*s))
        .collect();
    
    Json(StoryListResponse {
        count: stories.len(),
        stories,
    })
}

/// Create a new story
async fn create_story(
    State(state): State<AppState>,
    Json(req): Json<CreateStoryRequest>,
) -> Json<StoryResponse> {
    let setting = StorySetting::from_str(&req.setting)
        .unwrap_or(StorySetting::Fantasy);
    
    let mut engine = state.story_engine.lock().unwrap();
    
    // Check if we can create more stories
    if !engine.can_create() {
        return Json(StoryResponse {
            id: "".to_string(),
            title: "Maximum stories reached".to_string(),
            setting: "".to_string(),
            characters: vec![],
            location: "".to_string(),
            inventory: vec![],
            turn_count: 0,
            available_choices: vec![],
            created_at: 0,
            updated_at: 0,
        });
    }
    
    let story = engine.start_story(req.title, setting, req.characters);
    Json(StoryResponse::from(story))
}

/// Get a story by ID
async fn get_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<StoryResponse>> {
    let engine = state.story_engine.lock().unwrap();
    let story = engine.get_story(&id);
    Json(story.map(StoryResponse::from))
}

/// End a story
async fn end_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Option<StoryResponse>> {
    let mut engine = state.story_engine.lock().unwrap();
    let story = engine.end_story(&id);
    Json(story.as_ref().map(StoryResponse::from))
}

/// Make a choice in a story
async fn make_choice(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<MakeChoiceRequest>,
) -> Json<serde_json::Value> {
    let mut engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story_mut(&id) {
        let choice = story.make_choice(&req.choice_id);
        Json(serde_json::json!({
            "success": choice.is_some(),
            "choice": choice,
            "turn_count": story.turn_count,
        }))
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Story not found",
        }))
    }
}

/// Roll dice
async fn roll_dice(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RollDiceRequest>,
) -> Json<DiceRollResponse> {
    let result = DiceRoller::roll(&req.dice)
        .unwrap_or(DiceRoll {
            dice: req.dice.clone(),
            result: 0,
            description: "Error rolling dice".to_string(),
        });
    
    Json(DiceRollResponse {
        dice: result.dice,
        result: result.result,
        description: result.description,
    })
}

/// Advance story with new narrative
async fn advance_story(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AdvanceStoryRequest>,
) -> Json<serde_json::Value> {
    let mut engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story_mut(&id) {
        // Check turn limit
        if story.is_at_limit() {
            return Json(serde_json::json!({
                "success": false,
                "error": "Story has reached maximum turns",
            }));
        }
        
        let rolls = vec![];
        story.advance(req.narrative, req.choices, rolls);
        
        Json(serde_json::json!({
            "success": true,
            "turn_count": story.turn_count,
            "location": story.state.location,
        }))
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Story not found",
        }))
    }
}

/// Get available choices
async fn get_choices(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Vec<StoryChoice>> {
    let engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story(&id) {
        Json(story.state.available_choices.clone())
    } else {
        Json(vec![])
    }
}

/// Update story location
async fn update_location(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(location): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story_mut(&id) {
        if let Some(loc) = location.as_str() {
            story.set_location(loc.to_string());
            Json(serde_json::json!({
                "success": true,
                "location": story.state.location,
            }))
        } else {
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid location",
            }))
        }
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Story not found",
        }))
    }
}

/// Get inventory
async fn get_inventory(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<Vec<String>> {
    let engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story(&id) {
        Json(story.state.inventory.clone())
    } else {
        Json(vec![])
    }
}

/// Add item to inventory
async fn add_inventory(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(item): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut engine = state.story_engine.lock().unwrap();
    
    if let Some(story) = engine.get_story_mut(&id) {
        if let Some(item_str) = item.as_str() {
            story.state.add_item(item_str.to_string());
            Json(serde_json::json!({
                "success": true,
                "inventory": story.state.inventory,
            }))
        } else {
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid item",
            }))
        }
    } else {
        Json(serde_json::json!({
            "success": false,
            "error": "Story not found",
        }))
    }
}

/// Validate dice notation
async fn validate_dice(Query(params): Query<serde_json::Value>) -> Json<serde_json::Value> {
    let dice = params.get("dice")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    let result = DiceRoller::roll(dice);
    
    Json(serde_json::json!({
        "valid": result.is_ok(),
        "dice": dice,
        "error": result.err(),
    }))
}

impl AppState {
    /// Initialize story engine
    pub fn init_story_engine(&self) -> std::sync::Mutex<StoryEngine> {
        std::sync::Mutex::new(StoryEngine::new())
    }
}