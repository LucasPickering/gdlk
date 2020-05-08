import React, { useContext } from 'react';
import { IdeContext } from 'state/ide';
import graphql from 'babel-plugin-relay/macro';
import { makeStyles, Typography } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { IoInfo_programSpec } from './__generated__/IoInfo_programSpec.graphql';
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
  programSpec: IoInfo_programSpec;
  relay: RelayProp;
}> = ({ className, programSpec }) => {
  const localClasses = useLocalStyles();
  const { machineState } = useContext(IdeContext);

  return (
    <div className={clsx(localClasses.ioInfo, className)}>
      <Typography component="h3" variant="h6">
        I/O
      </Typography>

      <div className={localClasses.buffers}>
        <BufferDisplay
          label="Input"
          values={machineState?.input ?? programSpec.input}
          maxLength={programSpec.input.length}
        />
        <BufferDisplay
          label="Output"
          values={programSpec.expectedOutput}
          secondaryValues={machineState?.output ?? []}
          maxLength={programSpec.expectedOutput.length}
        />
      </div>
    </div>
  );
};

export default createFragmentContainer(IoInfo, {
  programSpec: graphql`
    fragment IoInfo_programSpec on ProgramSpecNode {
      input
      expectedOutput
    }
  `,
});
