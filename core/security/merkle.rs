// Minimal Merkle-audit trail scaffold for MVP
pub struct MerkleTrail {
    pub leaves: Vec<String>,
    pub hash: String,
}

impl MerkleTrail {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            hash: String::new(),
        }
    }
    pub fn append(&mut self, item: String) {
        self.leaves.push(item);
    }
    pub fn compute(&mut self) {
        self.hash = format!("hash({:?})", self.leaves);
    }
}
