pub struct AuditLog {
    entries: Vec<String>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            entries: Vec::new(),
        }
    }
    pub fn log(&mut self, entry: String) {
        self.entries.push(entry);
    }
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}
