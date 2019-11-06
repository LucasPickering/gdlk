use serde::{Deserialize, Serialize};

// Different stages of the AST. Each one has had certain transformations applied

// ===== Raw (parsed, with no checks/transformations applied) =====

#[derive(Debug, Serialize, Deserialize)]
pub enum RawValue {
    Int(u32),
}

pub type RawIdentifier = String;

#[derive(Debug, Serialize, Deserialize)]
pub enum RawInstruction {
    /// Sets the workspace to the given value
    Set(RawValue),
    /// Pushes the workspace onto the given stack
    Push(RawIdentifier),
    /// Pops the top value off the given stack into the workspace
    Pop(RawIdentifier),
    /// Creates a new stack with the given identifier
    Create(RawIdentifier),
    /// Destroys the stack with the given identifier
    Destroy(RawIdentifier),
}

/// The body of a program. In the future, could also represent the body of a
/// block or function.
pub type RawBody = Vec<RawInstruction>;

/// A program consists of a series of instructions
#[derive(Debug, Serialize, Deserialize)]
pub struct RawProgram {
    pub body: RawBody,
}

// ===== Well-formed (well-formedness checking has been applied) =====

pub type WellFormedValue = RawValue;
pub type WellFormedIdentifier = RawIdentifier;

#[derive(Debug, Serialize, Deserialize)]
pub enum WellFormedInstruction {
    /// Sets the workspace to the given value
    Set(WellFormedValue),
    /// Creates a new stack with the given identifier
    Create(WellFormedIdentifier),
    /// Destroys the stack with the given identifier
    Destroy(WellFormedIdentifier),
    /// Pushes the active value onto the given stack
    Push(WellFormedIdentifier),
    /// Pops the top value off the given stack into the active value
    Pop(WellFormedIdentifier),
}

pub type WellFormedBody = Vec<WellFormedInstruction>;

#[derive(Debug, Serialize, Deserialize)]
pub struct WellFormedProgram {
    pub body: WellFormedBody,
}
