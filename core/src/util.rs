use validator::{Validate, ValidationErrors};

/// A small wrapper struct to indicate that the wrapped value has been
/// validated. Built on top of <validate>. This struct cannot be constructed
/// except via <Self::try_from>.
///
/// ```
/// use gdlk::{HardwareSpec, Valid};
///
/// let maybe_valid = HardwareSpec {
///     num_registers: 1,
///     num_stacks: 1,
///     max_stack_length: 10,
/// };
/// let valid: Valid<HardwareSpec> = Valid::validate(maybe_valid).unwrap();
/// ```
pub struct Valid<T: Validate> {
    inner: T,
}

impl<T: Validate> Valid<T> {
    /// Validate the given value, and if validation succeeds, wrap it in a
    /// <Valid> to indicate it's valid.
    pub fn validate(value: T) -> Result<Self, ValidationErrors> {
        // We can't do a blanket TryFrom<T: Validate> implementation because of
        // this bug https://github.com/rust-lang/rust/issues/50133
        // Will have to wait for better specialization
        value.validate()?;
        Ok(Self { inner: value })
    }

    /// Get the inner value
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

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
                if ["1", "t", "true"]
                    .contains(&debug_val.to_lowercase().as_str())
                {
                    $arg
                }
            }
        }
    };
}
