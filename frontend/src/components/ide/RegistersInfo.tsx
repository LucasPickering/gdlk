import React, { useContext } from "react";
import { LangValue, IdeContext } from "@root/state/ide";
import { Typography } from "@mui/material";
import { makeStyles } from "@mui/styles";
import LangValueDisplay from "./LangValueDisplay";
import clsx from "clsx";

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  registers: {
    padding: spacing(1),
    backgroundColor: palette.background.default,
    display: "flex",
  },
  register: {
    display: "flex",
    flexDirection: "column",
    alignItems: "flex-end",
    "&:not(:first-child)": {
      paddingLeft: spacing(2),
    },
  },
}));

const RegisterDisplay: React.FC<{
  name: string;
  value?: LangValue;
}> = ({ name, value }) => {
  const localClasses = useLocalStyles();
  return (
    <div className={localClasses.register}>
      <Typography>{name}</Typography>
      <LangValueDisplay value={value} />
    </div>
  );
};

const RegistersInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { wasmHardwareSpec, compiledState } = useContext(IdeContext);
  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

  return (
    <div className={clsx(localClasses.registers, className)}>
      {wasmHardwareSpec.registers.map((name) => (
        <RegisterDisplay
          key={name}
          name={name}
          // If we're compiled, use the active values. Otherwise just show
          // the names with placeholder values.
          value={machineState?.registers[name]}
        />
      ))}
    </div>
  );
};

export default RegistersInfo;
