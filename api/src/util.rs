/// Macro that can wrap any body, and only executes the body if we are running
/// in debug mode. Debug mode is enabled by setting the environment variable
/// DEBUG=true. This compiles away to nothing when --release is used.
///
/// Example:
/// ```
/// debug!(println!("Hello!"));
/// ```
///
/// BTW that last assertion about --release hasn't _actually_ been confirmed,
/// feel free to test that yourself.
#[macro_export]
macro_rules! debug {
    ($arg:expr) => {
        #[cfg(debug_assertions)]
        {
            if let Ok(debug_val) = std::env::var("DEBUG") {
                if debug_val.to_lowercase().as_str() == "true" {
                    $arg
                }
            }
        }
    };
}

#[cfg(test)]
pub mod test {
    /// Asserts that a the first argument is an Err and that its error contains
    /// the string in the argument.
    #[macro_export]
    macro_rules! assert_error {
        ( $result:expr, $msg:tt ) => {
            match $result {
                Ok(_) => panic!("Expected Err, got Ok"),
                Err(error) => {
                    let error_str = error.to_string();
                    assert!(
                        error_str.contains($msg),
                        format!(
                            "Expected error \"{}\" to contain \"{}\"",
                            error_str, $msg
                        )
                    );
                }
            }
        };
    }
}
