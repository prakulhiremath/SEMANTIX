// SEMANTIX - Minimal CI-Passing Implementation

pub fn version() -> &'static str {
    "0.1.0"
}

pub fn estimate_cost(tokens: u32) -> u32 {
    tokens * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }

    #[test]
    fn test_estimate_cost() {
        assert_eq!(estimate_cost(100), 200);
    }
}
