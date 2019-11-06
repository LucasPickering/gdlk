use failure::Fail;

#[derive(Debug, Fail)]
pub enum CompilerError {
    /// Tried to declare an identifier that is already in scope
    #[fail(display = "Duplicate identifier")]
    DuplicateIdentifier(String),

    /// Tried to use an identifier that hasn't been declared
    #[fail(display = "Unknown identifier")]
    UnknownIdentifier(String),
}
