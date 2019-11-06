use crate::error::CompilerError;
use failure::Error;
use std::collections::HashSet;

/// An environment of identifiers
pub struct Env {
    idents: HashSet<String>,
}

impl Env {
    /// Creates a new empty environment
    pub fn new() -> Env {
        Env {
            idents: HashSet::new(),
        }
    }

    /// Declares the given identifier in this environment. If the identifier did
    /// not already exist in the environment, then it will be declared and
    /// return `Ok`. If it is already in the env, returns `Err`.
    pub fn declare(&mut self, ident: &str) -> Result<(), Error> {
        if self.idents.insert(ident.into()) {
            Ok(())
        } else {
            Err(CompilerError::DuplicateIdentifier(ident.into()).into())
        }
    }

    /// Verifies that the given identifier exists in this environment. If it
    /// does, returns `Ok`. If it does not, returns `Err`.
    pub fn verify_exists(&self, ident: &str) -> Result<(), Error> {
        if self.idents.contains(ident) {
            Ok(())
        } else {
            Err(CompilerError::UnknownIdentifier(ident.into()).into())
        }
    }
}
