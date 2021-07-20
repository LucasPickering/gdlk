import React, { useContext } from 'react';
import { IdeContext } from '@root/state/ide';
import { makeStyles } from '@material-ui/core';
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
  stack: {
    maxHeight: '100%',
    paddingLeft: '0 !important',
  },
}));

const StackInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { wasmHardwareSpec, compiledState } = useContext(IdeContext);
  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  return (
    <div className={clsx(localClasses.stackInfo, className)}>
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
  );
};

export default StackInfo;
