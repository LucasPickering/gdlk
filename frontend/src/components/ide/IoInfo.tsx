import React, { useContext } from 'react';
import { IdeContext } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  ioInfo: {
    backgroundColor: palette.background.default,
    padding: spacing(1),
  },
  buffers: {
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
    <div className={clsx(localClasses.ioInfo, className)}>
      <Typography component="h3" variant="h6">
        I/O
      </Typography>

      <div className={localClasses.buffers}>
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
    </div>
  );
};

export default IoInfo;
