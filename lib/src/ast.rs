// Different stages of the AST. Each one has had certain transformations applied

#[derive(Debug)]
pub enum Ast {
    Value(String),
}

/// Placeholder type right now
#[derive(Debug)]
pub enum WellFormedAst {
    Value(String),
}
