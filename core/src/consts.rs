/// The maximum number of cycles that a program can run for before being killed.
/// Programs that take this number of cycles WILL terminate normally, but going
/// over this number will cause an error when trying to execute additional
/// instructions.
pub const MAX_CYCLE_COUNT: usize = 1000;
