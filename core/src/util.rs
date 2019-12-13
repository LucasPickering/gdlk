/// Macro that can wrap any body, and only executes the body if we are running
/// in debug mode. Debug mode is enabled by setting the environment variable
/// DEBUG=true. This compiles away to nothing when --release is used.
///
/// Example:
/// ```
/// use gdlk::debug;
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
