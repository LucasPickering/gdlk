import React, { useContext } from 'react';
import { LangValue, IdeContext } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import { chain } from 'lodash';
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
  const { machineState } = useContext(IdeContext);
  const registers = machineState?.registers;

  return (
    <div className={clsx(localClasses.registersInfo, className)}>
      <Typography component="h3" variant="h6">
        Registers
      </Typography>

      <div className={localClasses.registers}>
        {registers ? (
          chain(registers)
            .toPairs()
            .sortBy(0)
            .map(([name, value]) => (
              <RegisterDisplay key={name} name={name} value={value} />
            ))
            .value()
        ) : (
          // Placeholder - TODO replace with all real registers once we can
          // pull that from the HardwareSpec
          <RegisterDisplay name="FAK" />
        )}
      </div>
    </div>
  );
};

export default RegistersInfo;
