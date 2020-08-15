//! Utilities for testing.

/// Assert that the first value is an Err, and that its string form matches
/// the second argument.
#[macro_export]
macro_rules! assert_err {
    ($res:expr, $msg:tt $(,)?) => {
        match $res {
            Ok(_) => panic!("Expected Err, got Ok"),
            Err(err) => assert_eq!(err.to_string(), $msg),
        }
    };
}
