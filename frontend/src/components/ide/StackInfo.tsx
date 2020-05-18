import React, { useContext } from 'react';
import { IdeContext } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  stackInfo: {
    padding: spacing(1),
    backgroundColor: palette.background.default,
  },
  stacks: {
    display: 'flex',
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
      <Typography component="h3" variant="h6">
        Stacks
      </Typography>

      <div className={localClasses.stacks}>
        {wasmHardwareSpec.stacks.map((name) => (
          <BufferDisplay
            key={name}
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
