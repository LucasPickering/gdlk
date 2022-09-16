import React, { useContext } from "react";
import { IdeContext } from "@root/state/ide";
import { Stack, Typography } from "@mui/material";
import LangValueDisplay from "./LangValueDisplay";

const RegistersInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const { wasmHardwareSpec, compiledState } = useContext(IdeContext);
  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

  return (
    <Stack className={className} direction="row" padding={1} spacing={1}>
      {wasmHardwareSpec.registers.map((name) => (
        <Stack key={name} direction="column" alignItems="end">
          <Typography>{name}</Typography>
          {/* If we're compiled, use the active values. Otherwise just show
              the names with placeholder values. */}
          <LangValueDisplay value={machineState?.registers[name]} />
        </Stack>
      ))}
    </Stack>
  );
};

export default RegistersInfo;
