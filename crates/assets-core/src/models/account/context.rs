use uuid::Uuid;

/// User context for filtering operations
#[derive(Debug, Clone, PartialEq)]
pub enum UserContext {
    User(Uuid), // Specific user view
    Family,     // Combined family view
}
