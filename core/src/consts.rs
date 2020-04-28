/// The maximum number of cycles that a program can run for before being killed.
/// Programs that take this number of cycles WILL terminate normally, but going
/// over this number will cause an error when trying to execute additional
/// instructions. This isn't meant to be a strategic restriction on users, just
/// a mechanism to prevent programs from running forever.
pub const MAX_CYCLE_COUNT: usize = 1_000_000;

/// The prefix that indicates a stack reference.
pub const STACK_REF_TAG: &str = "S";
/// The string that refers to the input length register.
pub const INPUT_LENGTH_REGISTER_REF: &str = "RLI";
/// The prefix that indicates a reference to a stack length register.
pub const STACK_LENGTH_REGISTER_REF_TAG: &str = "RS";
/// The prefix that indicates a reference to a user register.
pub const USER_REGISTER_REF_TAG: &str = "RX";
