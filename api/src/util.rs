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
