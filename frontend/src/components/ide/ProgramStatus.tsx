import React, { useContext } from "react";
import { CompiledState, IdeContext } from "@root/state/ide";
import clsx from "clsx";
import { makeStyles } from "@material-ui/core";
const { FailureReason } = await import("gdlk_wasm");

const useLocalStyles = makeStyles(({ spacing }) => ({
  programStatus: {
    padding: spacing(1),
  },
}));

function getStatusText(
  compiledState: CompiledState | undefined,
  stepping: boolean
): string {
  // Compilation hasn't been attempted, so the source is probably empty
  if (!compiledState) {
    return "";
  }

  // Program failed to compile
  if (compiledState.type === "error") {
    return "Error - Compilation failure";
  }

  const machineState = compiledState.machineState;

  // Compiled, ready to execute
  if (!machineState.terminated && !stepping) {
    return "Ready";
  }
  // Compiled and executing
  if (!machineState.terminated && stepping) {
    return "Running...";
  }

  // Terminated successfully!
  if (machineState.successful) {
    return "Success";
  }

  // Give some detail on why the program failed
  switch (machineState.failureReason) {
    case FailureReason.RuntimeError:
      return `Error - ${machineState.runtimeError?.text}`;
    case FailureReason.RemainingInput:
      return `Failure - Values remain in input buffer`;
    case FailureReason.IncorrectOutput:
      return `Failure - Output did not match expectation`;
    default:
      // If we hit this, we either got an unexpected enum value from wasm, or
      // the failure reason is undefined which shouldn't be possible since we
      // checked the success case already
      // eslint-disable-next-line no-console
      console.error(
        `Unexpected program failure state: ${machineState.failureReason}`
      );
      return "Failure";
  }
}

const ProgramStatus: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { stepping, compiledState } = useContext(IdeContext);

  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

  return (
    <div className={clsx(className, localClasses.programStatus)}>
      <div>CPU Cycles: {machineState?.cycleCount ?? "–"}</div>
      <div>{getStatusText(compiledState, stepping)}</div>
    </div>
  );
};

export default ProgramStatus;
