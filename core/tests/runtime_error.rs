//! Integration tests for GDLK that expect compile errors. The programs in
//! these tests should all fail during compilation.

use gdlk::{compile, HardwareSpec, ProgramSpec};

/// Compiles the program for the given hardware, executes it under the given
/// program spec, and expects a runtime error. Panics if the program executes
/// successfully, or if the wrong set of errors is returned.
fn expect_runtime_error(
    hardware_spec: HardwareSpec,
    program_spec: ProgramSpec,
    src: &str,
    expected_error: &str,
) {
    // Compile from hardware+src
    let mut machine =
        compile(&hardware_spec, &program_spec, src.into()).unwrap();

    // Execute to completion
    let actual_error = machine.execute_all().unwrap_err();
    assert_eq!(format!("{}", actual_error), expected_error);
}

#[test]
fn test_exceed_max_cycle_count() {
    // We can exit successfully with exactly the maximum number of cycles
    expect_runtime_error(
        HardwareSpec::default(),
        ProgramSpec {
            input: vec![],
            expected_output: vec![],
        },
        "
        START:
        JMP START
        ",
        "The maximum number of cycles has been reached",
    );
}
