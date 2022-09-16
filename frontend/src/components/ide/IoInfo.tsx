import React, { useContext } from "react";
import { IdeContext } from "@root/state/ide";
import BufferDisplay from "./BufferDisplay";
import { Divider, Stack } from "@mui/material";

const IoInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const { wasmProgramSpec, compiledState } = useContext(IdeContext);

  const input = Array.from(wasmProgramSpec.input);
  const expectedOutput = Array.from(wasmProgramSpec.expectedOutput);
  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

  return (
    <Stack className={className} direction="column" padding={1}>
      <BufferDisplay
        direction="row"
        label="Input"
        values={machineState?.input ?? input}
        maxLength={input.length}
        sx={{ flex: 1 }}
      />
      <Divider />
      <BufferDisplay
        direction="row"
        label="Output"
        values={machineState?.output ?? []}
        maxLength={expectedOutput.length}
        sx={{ flex: 1 }}
      />
    </Stack>
  );
};

export default IoInfo;
