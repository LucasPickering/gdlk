import React, { useContext } from "react";
import { IdeContext } from "@root/state/ide";
import BufferDisplay from "./BufferDisplay";
import { Stack } from "@mui/material";

const StackInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const { wasmHardwareSpec, compiledState } = useContext(IdeContext);
  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

  return (
    <Stack className={className} direction="row" padding={1} spacing={1}>
      {wasmHardwareSpec.stacks.map((name) => (
        <BufferDisplay
          key={name}
          // Reverse the column so values are inserted bottom-up
          direction="column-reverse"
          label={name}
          values={machineState?.stacks[name] ?? []}
          maxLength={wasmHardwareSpec.max_stack_length}
        />
      ))}
    </Stack>
  );
};

export default StackInfo;
