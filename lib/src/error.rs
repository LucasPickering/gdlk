use failure::Fail;

#[derive(Debug, Fail)]
pub enum CompilerError {
    // placeholder error, until we have real stuff for this
    #[fail(display = "shit!")]
    ShitError,
}
