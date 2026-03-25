//! Story Engine - Interactive fiction framework
//!
//! Feature 7: Story Engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::unified_config::story_constants::*;

/// Story setting/genre
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorySetting {
    Fantasy,
    Scifi,
    Horror,
    Mystery,
    Western,
    Modern,
}

impl Default for StorySetting {
    fn default() -> Self {
        StorySetting::Fantasy
    }
}

impl StorySetting {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fantasy" => Some(StorySetting::Fantasy),
            "scifi" => Some(StorySetting::Scifi),
            "horror" => Some(StorySetting::Horror),
            "mystery" => Some(StorySetting::Mystery),
            "western" => Some(StorySetting::Western),
            "modern" => Some(StorySetting::Modern),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            StorySetting::Fantasy => "fantasy",
            StorySetting::Scifi => "scifi",
            StorySetting::Horror => "horror",
            StorySetting::Mystery => "mystery",
            StorySetting::Western => "western",
            StorySetting::Modern => "modern",
        }
    }
}

/// A character in the story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryCharacter {
    /// Character name
    pub name: String,
    /// Character description/role
    pub description: String,
    /// Character's current HP (for games)
    pub hp: Option<u32>,
    /// Character's attributes
    pub attributes: HashMap<String, i32>,
}

/// NPC state in story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcState {
    /// NPC name
    pub name: String,
    /// Whether NPC is present
    pub present: bool,
    /// NPC disposition (friendly, neutral, hostile)
    pub disposition: String,
    /// Custom data
    pub data: HashMap<String, String>,
}

/// A story beat (one turn's narrative)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryBeat {
    /// Turn number
    pub turn: u32,
    /// Narrative text
    pub narrative: String,
    /// Choices presented
    pub choices: Vec<StoryChoice>,
    /// Dice rolls this turn
    pub dice_rolls: Vec<DiceRoll>,
    /// Timestamp
    pub timestamp: i64,
}

/// A choice presented to the player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryChoice {
    /// Choice ID
    pub id: String,
    /// Choice text
    pub text: String,
    /// Consequence description
    pub consequence: Option<String>,
}

/// Dice roll result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceRoll {
    /// Dice notation (e.g., "2d6+3")
    pub dice: String,
    /// Result value
    pub result: u32,
    /// Description of what was rolled for
    pub description: String,
}

/// Current story state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryState {
    /// Current location
    pub location: String,
    /// Player inventory
    pub inventory: Vec<String>,
    /// NPCs in current scene
    pub npcs: Vec<NpcState>,
    /// Story flags (custom state)
    pub flags: HashMap<String, bool>,
    /// Turn history
    pub history: Vec<StoryBeat>,
    /// Current available choices
    pub available_choices: Vec<StoryChoice>,
}

impl Default for StoryState {
    fn default() -> Self {
        Self {
            location: "Unknown".to_string(),
            inventory: Vec::new(),
            npcs: Vec::new(),
            flags: HashMap::new(),
            history: Vec::new(),
            available_choices: Vec::new(),
        }
    }
}

impl StoryState {
    /// Add an item to inventory
    pub fn add_item(&mut self, item: String) {
        if self.inventory.len() < MAX_INVENTORY_ITEMS && !self.inventory.contains(&item) {
            self.inventory.push(item);
        }
    }

    /// Remove an item from inventory
    pub fn remove_item(&mut self, item: &str) -> bool {
        if let Some(pos) = self.inventory.iter().position(|i| i == item) {
            self.inventory.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if player has item
    pub fn has_item(&self, item: &str) -> bool {
        self.inventory.contains(&item.to_string())
    }

    /// Set a flag
    pub fn set_flag(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_string(), value);
    }

    /// Get a flag
    pub fn get_flag(&self, key: &str) -> bool {
        *self.flags.get(key).unwrap_or(&false)
    }

    /// Add a story beat to history
    pub fn add_beat(&mut self, beat: StoryBeat) {
        self.history.push(beat);
    }

    /// Update available choices
    pub fn set_choices(&mut self, choices: Vec<StoryChoice>) {
        self.available_choices = choices;
    }
}

/// Complete story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    /// Unique story ID
    pub id: String,
    /// Story title
    pub title: String,
    /// Setting/genre
    pub setting: StorySetting,
    /// Characters in the story
    pub characters: Vec<StoryCharacter>,
    /// Current state
    pub state: StoryState,
    /// Number of turns taken
    pub turn_count: u32,
    /// Optional linked task ID
    pub task_id: Option<String>,
    /// When story was created
    pub created_at: i64,
    /// Last update time
    pub updated_at: i64,
}

impl Story {
    /// Create a new story
    pub fn new(title: String, setting: StorySetting, characters: Vec<StoryCharacter>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            setting,
            characters,
            state: StoryState::default(),
            turn_count: 0,
            task_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a simple story with just a title
    pub fn simple(title: &str) -> Self {
        Self::new(title.to_string(), StorySetting::default(), Vec::new())
    }

    /// Advance the story by one turn
    pub fn advance(
        &mut self,
        narrative: String,
        choices: Vec<StoryChoice>,
        rolls: Vec<DiceRoll>,
    ) -> StoryBeat {
        let beat = StoryBeat {
            turn: self.turn_count,
            narrative,
            choices: choices.clone(),
            dice_rolls: rolls,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        self.state.add_beat(beat.clone());
        self.state.set_choices(choices);
        self.turn_count += 1;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        beat
    }

    /// Make a choice
    pub fn make_choice(&mut self, choice_id: &str) -> Option<&StoryChoice> {
        self.state
            .available_choices
            .iter()
            .find(|c| c.id == choice_id)
    }

    /// Update location
    pub fn set_location(&mut self, location: String) {
        self.state.location = location;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Check if story is at turn limit
    pub fn is_at_limit(&self) -> bool {
        self.turn_count >= MAX_STORY_TURNS as u32
    }
}

/// Dice roller
pub struct DiceRoller;

impl DiceRoller {
    /// Roll dice (e.g., "2d6+3")
    pub fn roll(notation: &str) -> Result<DiceRoll, String> {
        // Parse notation like "2d6+3" or "d20"
        let notation = notation.trim().to_lowercase();

        // Extract parts
        let (count, sides, modifier) = Self::parse_notation(&notation)?;

        // Roll each die
        let mut results: Vec<u32> = Vec::with_capacity(count);
        for _ in 0..count {
            // Simple pseudo-random (in production use proper RNG)
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u32;
            let roll = (now % sides as u32) + 1;
            results.push(roll);
        }

        let sum: u32 = results.iter().sum();
        let total = sum.wrapping_add(modifier as u32);

        Ok(DiceRoll {
            dice: notation.clone(),
            result: total,
            description: format!("{} = {} + {}", notation, sum, modifier),
        })
    }

    /// Parse dice notation into (count, sides, modifier)
    fn parse_notation(notation: &str) -> Result<(usize, u32, i32), String> {
        let mut count = 1;
        let mut sides = 20;
        let mut modifier = 0;

        // Must contain 'd' for valid dice notation
        if !notation.contains('d') {
            return Err("Invalid notation: must contain 'd'".to_string());
        }

        // Handle "d20" format (count = 1)
        if notation.starts_with("d") {
            sides = notation[1..]
                .trim_end_matches(|c: char| c.is_alphabetic())
                .parse()
                .map_err(|_| "Invalid dice sides".to_string())?;
            if sides < 1 || sides > 1000 {
                return Err("Dice sides must be between 1 and 1000".to_string());
            }
            return Ok((count, sides, modifier));
        }

        // Handle full notation like "2d6+3"
        let parts: Vec<&str> = notation
            .split(|c| c == 'd' || c == '+' || c == '-')
            .collect();

        if parts.is_empty() {
            return Err("Invalid notation".to_string());
        }

        // Count
        if let Some(c) = parts.first() {
            if !c.is_empty() {
                count = c.parse().map_err(|_| "Invalid dice count")?;
                if count > 100 {
                    return Err("Too many dice (max 100)".to_string());
                }
            }
        }

        // Sides
        if parts.len() > 1 {
            if let Some(s) = parts.get(1) {
                if !s.is_empty() {
                    sides = s.parse().unwrap_or(20);
                    if sides < 1 || sides > 1000 {
                        return Err("Dice sides must be between 1 and 1000".to_string());
                    }
                }
            }
        }

        // Modifier
        if parts.len() > 2 {
            if let Some(m) = parts.get(2) {
                if !m.is_empty() {
                    // Check if it was + or -
                    let sign = if notation.contains("-") { -1 } else { 1 };
                    modifier = sign * m.parse::<i32>().unwrap_or(0);
                }
            }
        }

        Ok((count, sides, modifier))
    }
}

/// Story engine - manages multiple stories
pub struct StoryEngine {
    /// Active stories
    stories: HashMap<String, Story>,
}

impl Default for StoryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StoryEngine {
    /// Create a new story engine
    pub fn new() -> Self {
        Self {
            stories: HashMap::new(),
        }
    }

    /// Start a new story
    pub fn start_story(
        &mut self,
        title: String,
        setting: StorySetting,
        characters: Vec<StoryCharacter>,
    ) -> &Story {
        let story = Story::new(title, setting, characters);
        let id = story.id.clone();
        self.stories.insert(id.clone(), story);
        self.stories.get(&id).unwrap()
    }

    /// Get a story by ID
    pub fn get_story(&self, id: &str) -> Option<&Story> {
        self.stories.get(id)
    }

    /// Get mutable story
    pub fn get_story_mut(&mut self, id: &str) -> Option<&mut Story> {
        self.stories.get_mut(id)
    }

    /// End a story
    pub fn end_story(&mut self, id: &str) -> Option<Story> {
        self.stories.remove(id)
    }

    /// List all active stories
    pub fn list_stories(&self) -> Vec<&Story> {
        self.stories.values().collect()
    }

    /// Count of active stories
    pub fn story_count(&self) -> usize {
        self.stories.len()
    }

    /// Check if can create more stories
    pub fn can_create(&self) -> bool {
        self.stories.len() < MAX_STORIES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_creation() {
        let story = Story::simple("Test Story");
        assert_eq!(story.title, "Test Story");
        assert_eq!(story.turn_count, 0);
    }

    #[test]
    fn test_story_with_characters() {
        let chars = vec![StoryCharacter {
            name: "Hero".to_string(),
            description: "The protagonist".to_string(),
            hp: Some(10),
            attributes: HashMap::new(),
        }];
        let story = Story::new("Epic Tale".to_string(), StorySetting::Fantasy, chars);
        assert_eq!(story.characters.len(), 1);
        assert_eq!(story.characters[0].name, "Hero");
    }

    #[test]
    fn test_story_advance() {
        let mut story = Story::simple("Test");
        let choices = vec![StoryChoice {
            id: "1".to_string(),
            text: "Go left".to_string(),
            consequence: None,
        }];
        let rolls = vec![];

        let beat = story.advance("Narrative text".to_string(), choices.clone(), rolls);

        assert_eq!(beat.turn, 0);
        assert_eq!(beat.narrative, "Narrative text");
        assert_eq!(story.turn_count, 1);
    }

    #[test]
    fn test_story_state_inventory() {
        let mut state = StoryState::default();
        state.add_item("Sword".to_string());
        assert!(state.has_item("Sword"));
        assert!(!state.has_item("Shield"));

        state.remove_item("Sword");
        assert!(!state.has_item("Sword"));
    }

    #[test]
    fn test_story_state_flags() {
        let mut state = StoryState::default();
        state.set_flag("found_treasure", true);
        assert!(state.get_flag("found_treasure"));
        assert!(!state.get_flag("unknown"));
    }

    #[test]
    fn test_dice_roller() {
        let result = DiceRoller::roll("d6").unwrap();
        assert!(result.result >= 1 && result.result <= 6);

        let result2 = DiceRoller::roll("2d6+3").unwrap();
        assert!(result2.result >= 5 && result2.result <= 15);
    }

    #[test]
    fn test_dice_roller_invalid() {
        let result = DiceRoller::roll("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_story_engine() {
        let mut engine = StoryEngine::new();

        assert!(engine.can_create());

        let chars = vec![];
        engine.start_story("Tale".to_string(), StorySetting::Fantasy, chars);

        assert_eq!(engine.story_count(), 1);

        let stories = engine.list_stories();
        assert_eq!(stories.len(), 1);
    }

    #[test]
    fn test_story_setting_conversion() {
        assert_eq!(
            StorySetting::from_str("fantasy"),
            Some(StorySetting::Fantasy)
        );
        assert_eq!(StorySetting::from_str("scifi"), Some(StorySetting::Scifi));
        assert_eq!(StorySetting::from_str("unknown"), None);

        assert_eq!(StorySetting::Fantasy.as_str(), "fantasy");
    }

    #[test]
    fn test_story_location() {
        let mut story = Story::simple("Test");
        story.set_location("Dark Cave".to_string());
        assert_eq!(story.state.location, "Dark Cave");
    }

    #[test]
    fn test_story_at_limit() {
        let mut story = Story::simple("Test");
        for _ in 0..MAX_STORY_TURNS {
            story.advance("text".to_string(), vec![], vec![]);
        }
        assert!(story.is_at_limit());
    }
}
