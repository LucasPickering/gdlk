import React, { useContext } from 'react';
import { IdeContext } from '@root/state/ide';
import { makeStyles } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  ioBuffers: {
    backgroundColor: palette.background.default,
    padding: spacing(1),
    display: 'flex',
  },
}));

const IoInfo: React.FC<{
  className?: string;
}> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { wasmProgramSpec, compiledState } = useContext(IdeContext);

  const input = Array.from(wasmProgramSpec.input);
  const expectedOutput = Array.from(wasmProgramSpec.expectedOutput);
  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  return (
    <div className={clsx(localClasses.ioBuffers, className)}>
      <BufferDisplay
        label="Input"
        values={machineState?.input ?? input}
        maxLength={input.length}
      />
      <BufferDisplay
        label="Output"
        values={expectedOutput}
        secondaryValues={machineState?.output ?? []}
        maxLength={expectedOutput.length}
      />
    </div>
  );
};

export default IoInfo;
