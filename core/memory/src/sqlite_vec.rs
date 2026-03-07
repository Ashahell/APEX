#[cfg(feature = "vector-search")]
pub mod vec {
    use rusqlite::ffi::sqlite3_auto_extension;
    use rusqlite::{params, Connection, Result as RusqliteResult};
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::sync::Mutex;
    use std::sync::OnceLock;

    static EXTENSION_REGISTERED: OnceLock<()> = OnceLock::new();

    fn register_vec_extension() {
        EXTENSION_REGISTERED.get_or_init(|| unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        });
    }

    fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(v.len() * 4);
        for f in v {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        bytes
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VecSearchResult {
        pub chunk_id: String,
        pub distance: f64,
        pub rowid: i64,
    }

    pub struct VecSearcher {
        conn: Mutex<Connection>,
        dimension: usize,
        next_id: Mutex<i64>,
    }

    impl VecSearcher {
        pub fn new(db_path: &Path, dimension: usize) -> RusqliteResult<Self> {
            register_vec_extension();
            let conn = Connection::open(db_path)?;
            Self::from_connection(conn, dimension)
        }

        pub fn from_connection(conn: Connection, dimension: usize) -> RusqliteResult<Self> {
            Ok(Self {
                conn: Mutex::new(conn),
                dimension,
                next_id: Mutex::new(1),
            })
        }

        pub fn initialize(&self) -> RusqliteResult<()> {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                &format!(
                    "CREATE VIRTUAL TABLE IF NOT EXISTS memory_vec0 USING vec0(embedding float[{}], chunk_id text)",
                    self.dimension
                ),
                [],
            )?;
            Ok(())
        }

        pub fn insert_embedding(&self, chunk_id: &str, embedding: &[f32]) -> RusqliteResult<()> {
            let conn = self.conn.lock().unwrap();
            let embedding_bytes = vec_to_bytes(embedding);
            conn.execute(
                "INSERT INTO memory_vec0(rowid, embedding, chunk_id) VALUES (NULL, ?, ?)",
                params![embedding_bytes, chunk_id],
            )?;
            Ok(())
        }

        pub fn search(&self, query: &[f32], k: usize) -> RusqliteResult<Vec<VecSearchResult>> {
            let conn = self.conn.lock().unwrap();
            let query_bytes = vec_to_bytes(query);
            let mut stmt = conn.prepare(
                "SELECT chunk_id, distance FROM memory_vec0 WHERE embedding match ?1 ORDER BY distance LIMIT ?2"
            )?;

            let results = stmt.query_map(params![query_bytes, k as i64], |row| {
                Ok(VecSearchResult {
                    chunk_id: row.get(0)?,
                    distance: row.get(1)?,
                    rowid: 0,
                })
            })?;

            results.collect()
        }

        pub fn search_with_filter(
            &self,
            query: &[f32],
            k: usize,
            filter_sql: &str,
        ) -> RusqliteResult<Vec<VecSearchResult>> {
            let conn = self.conn.lock().unwrap();
            let query_bytes = vec_to_bytes(query);
            let sql = format!(
                "SELECT chunk_id, distance FROM memory_vec0 WHERE embedding match ?1 AND {} ORDER BY distance LIMIT ?2",
                filter_sql
            );
            let mut stmt = conn.prepare(&sql)?;

            let results = stmt.query_map(params![query_bytes, k as i64], |row| {
                Ok(VecSearchResult {
                    chunk_id: row.get(0)?,
                    distance: row.get(1)?,
                    rowid: 0,
                })
            })?;

            results.collect()
        }

        pub fn delete(&self, chunk_id: &str) -> RusqliteResult<()> {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "DELETE FROM memory_vec0 WHERE chunk_id = ?1",
                params![chunk_id],
            )?;
            Ok(())
        }

        pub fn count(&self) -> RusqliteResult<i64> {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM memory_vec0", [], |row| row.get(0))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

        #[test]
        fn test_vec_searcher() -> RusqliteResult<()> {
            let temp_dir = std::env::temp_dir();
            let db_path = temp_dir.join("test_vec.db");
            let _ = fs::remove_file(&db_path);

            {
                let searcher = VecSearcher::new(&db_path, 4)?;
                searcher.initialize()?;

                searcher.insert_embedding("chunk1", &[0.1, 0.1, 0.1, 0.1])?;
                searcher.insert_embedding("chunk2", &[0.2, 0.2, 0.2, 0.2])?;
                searcher.insert_embedding("chunk3", &[0.3, 0.3, 0.3, 0.3])?;

                let results = searcher.search(&[0.25, 0.25, 0.25, 0.25], 2)?;

                assert_eq!(results.len(), 2);
                assert!(results[0].distance <= results[1].distance);
            }

            let _ = fs::remove_file(&db_path);
            Ok(())
        }
    }
}
