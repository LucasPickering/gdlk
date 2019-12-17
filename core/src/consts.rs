/// The name of the register that holds the current input length
pub const REG_INPUT_LEN: &str = "RLI";
/// The prefix for all registers that hold stack lengths (e.g. "RS0", "RS1")
pub const REG_STACK_LEN_PREFIX: &str = "RS";
/// The prefix for all user registers (e.g. "RX0")
pub const REG_USER_PREFIX: &str = "RX";

/// The maximum number of cycles that a program can run for before being killed.
/// Programs that take this number of cycles WILL terminate normally, but going
/// over this number will cause an error when trying to execute additional
/// instructions.
pub const MAX_CYCLE_COUNT: usize = 1000;
