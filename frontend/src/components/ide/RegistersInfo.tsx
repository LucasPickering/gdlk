import React, { useContext } from 'react';
import { LangValue, IdeContext } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import LangValueDisplay from './LangValueDisplay';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  registersInfo: {
    padding: spacing(1),
    backgroundColor: palette.background.default,
  },
  registers: {
    display: 'flex',
  },
  register: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    '&:not(:first-child)': {
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
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  return (
    <div className={clsx(localClasses.registersInfo, className)}>
      <Typography variant="h3">Registers</Typography>

      <div className={localClasses.registers}>
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
    </div>
  );
};

export default RegistersInfo;
