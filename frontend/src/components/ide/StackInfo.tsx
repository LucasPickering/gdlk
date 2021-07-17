import React, { useContext } from 'react';
import { IdeContext } from '@root/state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  stackInfo: {
    display: 'flex',
    flexDirection: 'column',
    padding: spacing(1),
    backgroundColor: palette.background.default,
    height: '100%',
  },
  stacks: {
    display: 'flex',
    flexGrow: 1,
    maxHeight: '100%',
  },
  stack: {
    maxHeight: '100%',
  },
}));

const StackInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { wasmHardwareSpec, compiledState } = useContext(IdeContext);
  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  if (wasmHardwareSpec.num_stacks === 0) {
    return null;
  }

  return (
    <div className={clsx(localClasses.stackInfo, className)}>
      <Typography variant="h3">Stacks</Typography>

      <div className={localClasses.stacks}>
        {wasmHardwareSpec.stacks.map((name) => (
          <BufferDisplay
            className={localClasses.stack}
            key={name}
            invert
            label={name}
            values={machineState?.stacks[name] ?? []}
            maxLength={wasmHardwareSpec.max_stack_length}
          />
        ))}
      </div>
    </div>
  );
};

export default StackInfo;
