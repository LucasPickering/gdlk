import React, { useContext } from 'react';
import { IdeContext } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import BufferDisplay from './BufferDisplay';
import { chain } from 'lodash';
import graphql from 'babel-plugin-relay/macro';
import { RelayProp, createFragmentContainer } from 'react-relay';
import { StackInfo_hardwareSpec } from './__generated__/StackInfo_hardwareSpec.graphql';
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
  hardwareSpec: StackInfo_hardwareSpec;
  relay: RelayProp;
}> = ({ className, hardwareSpec }) => {
  const localClasses = useLocalStyles();
  const { machineState } = useContext(IdeContext);
  const stacks = machineState?.stacks ?? {};

  return (
    <div className={clsx(localClasses.stackInfo, className)}>
      <Typography component="h3" variant="h6">
        Stacks
      </Typography>

      <div className={localClasses.stacks}>
        {chain(stacks)
          .toPairs()
          .sortBy(0)
          .map(([name, values]) => (
            <BufferDisplay
              key={name}
              label={name}
              values={values}
              maxLength={hardwareSpec.maxStackLength}
            />
          ))
          .value()}
      </div>
    </div>
  );
};

export default createFragmentContainer(StackInfo, {
  hardwareSpec: graphql`
    fragment StackInfo_hardwareSpec on HardwareSpecNode {
      maxStackLength
    }
  `,
});
