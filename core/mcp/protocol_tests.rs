#[cfg(test)]
mod tests {
    use super::super::protocol as _; // ensure module exists
    use crate::core::mcp::registry;

    #[test]
    fn bootstrap_and_list_tools() {
        // Bootstrap a couple of tools and ensure list_tools has items
        registry::bootstrap_two_tools();
        let list = registry::list_tools();
        assert!(list.len() >= 2);
    }
}
