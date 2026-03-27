use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub attributes: serde_json::Value,
    pub first_seen: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    pub cause: String,
    pub effect: String,
    pub timestamp: DateTime<Utc>,
}

pub struct WorkingMemory {
    task_id: String,
    scratchpad: String,
    active_entities: HashMap<String, Entity>,
    causal_links: Vec<CausalLink>,
    created_at: DateTime<Utc>,
    pool: Pool<Sqlite>,
}

impl WorkingMemory {
    pub async fn new(task_id: &str, pool: Pool<Sqlite>) -> Result<Self, sqlx::Error> {
        if let Some(saved) = Self::restore(task_id, &pool).await? {
            return Ok(saved);
        }

        Ok(Self {
            task_id: task_id.to_string(),
            scratchpad: String::new(),
            active_entities: HashMap::new(),
            causal_links: Vec::new(),
            created_at: Utc::now(),
            pool,
        })
    }

    pub async fn restore(task_id: &str, pool: &Pool<Sqlite>) -> Result<Option<Self>, sqlx::Error> {
        let row: Option<(String, String, String, String)> = sqlx::query_as(
            "SELECT task_id, scratchpad, entities_json, causal_links_json FROM working_memory WHERE task_id = ?"
        )
        .bind(task_id)
        .fetch_optional(pool)
        .await?;

        if let Some((task_id, scratchpad, entities_json, causal_links_json)) = row {
            let active_entities: HashMap<String, Entity> =
                serde_json::from_str(&entities_json).unwrap_or_default();
            let causal_links: Vec<CausalLink> =
                serde_json::from_str(&causal_links_json).unwrap_or_default();

            Ok(Some(Self {
                task_id,
                scratchpad,
                active_entities,
                causal_links,
                created_at: Utc::now(),
                pool: pool.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_scratchpad(&mut self, text: &str) -> Result<(), sqlx::Error> {
        self.scratchpad = text.to_string();
        self.persist().await
    }

    pub fn get_scratchpad(&self) -> &str {
        &self.scratchpad
    }

    pub async fn add_entity(&mut self, entity: Entity) -> Result<(), sqlx::Error> {
        self.active_entities.insert(entity.name.clone(), entity);
        self.persist().await
    }

    pub fn get_entities(&self) -> &HashMap<String, Entity> {
        &self.active_entities
    }

    pub async fn add_causal_link(&mut self, cause: &str, effect: &str) -> Result<(), sqlx::Error> {
        self.causal_links.push(CausalLink {
            cause: cause.to_string(),
            effect: effect.to_string(),
            timestamp: Utc::now(),
        });
        self.persist().await
    }

    pub fn get_causal_links(&self) -> &[CausalLink] {
        &self.causal_links
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    async fn persist(&self) -> Result<(), sqlx::Error> {
        let entities_json = serde_json::to_string(&self.active_entities).unwrap_or_default();
        let causal_links_json = serde_json::to_string(&self.causal_links).unwrap_or_default();

        sqlx::query(
            "INSERT OR REPLACE INTO working_memory
             (task_id, scratchpad, entities_json, causal_links_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.task_id)
        .bind(&self.scratchpad)
        .bind(&entities_json)
        .bind(&causal_links_json)
        .bind(self.created_at.to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn flush_to_longterm(&self, _file_path: &str) -> Result<String, sqlx::Error> {
        let narrative = self.generate_narrative();
        Ok(narrative)
    }

    fn generate_narrative(&self) -> String {
        let mut narrative = String::new();

        narrative.push_str(&format!("# Task Working Memory: {}\n\n", self.task_id));
        narrative.push_str(&format!(
            "**Created**: {}\n\n",
            self.created_at.format("%Y-%m-%d %H:%M UTC")
        ));

        narrative.push_str("## Scratchpad\n\n");
        if self.scratchpad.is_empty() {
            narrative.push_str("_No scratchpad entries_\n\n");
        } else {
            narrative.push_str(&self.scratchpad);
            narrative.push_str("\n\n");
        }

        if !self.active_entities.is_empty() {
            narrative.push_str("## Active Entities\n\n");
            for entity in self.active_entities.values() {
                narrative.push_str(&format!(
                    "- **{}** ({}): {:?}\n",
                    entity.name, entity.entity_type, entity.attributes
                ));
            }
            narrative.push_str("\n");
        }

        if !self.causal_links.is_empty() {
            narrative.push_str("## Causal Links\n\n");
            for link in &self.causal_links {
                narrative.push_str(&format!("- {} → {}\n", link.cause, link.effect));
            }
            narrative.push_str("\n");
        }

        narrative
    }

    pub async fn delete(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM working_memory WHERE task_id = ?")
            .bind(&self.task_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_serialization() {
        let entity = Entity {
            name: "test".to_string(),
            entity_type: "project".to_string(),
            attributes: serde_json::json!({"status": "active"}),
            first_seen: Utc::now(),
            last_updated: Utc::now(),
        };

        let serialized = serde_json::to_string(&entity).unwrap();
        let deserialized: Entity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(entity.name, deserialized.name);
        assert_eq!(entity.entity_type, deserialized.entity_type);
    }

    #[test]
    fn test_causal_link_serialization() {
        let link = CausalLink {
            cause: "action_a".to_string(),
            effect: "result_b".to_string(),
            timestamp: Utc::now(),
        };

        let serialized = serde_json::to_string(&link).unwrap();
        let deserialized: CausalLink = serde_json::from_str(&serialized).unwrap();

        assert_eq!(link.cause, deserialized.cause);
        assert_eq!(link.effect, deserialized.effect);
    }
}
