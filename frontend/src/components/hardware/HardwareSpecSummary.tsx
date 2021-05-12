import React from 'react';
import { RelayProp, useFragment } from 'react-relay';
import { graphql } from 'react-relay';
import { HardwareSpecSummary_hardwareSpec$key } from './__generated__/HardwareSpecSummary_hardwareSpec.graphql';
import SimpleTable from 'components/common/SimpleTable';

const HardwareSpecSummary: React.FC<{
  hardwareSpecKey: HardwareSpecSummary_hardwareSpec$key;
}> = ({ hardwareSpecKey }) => {
  const hardwareSpec = useFragment(
    graphql`
      fragment HardwareSpecSummary_hardwareSpec on HardwareSpecNode {
        numRegisters
        numStacks
        maxStackLength
      }
    `,
    hardwareSpecKey
  );

  return (
    <SimpleTable
      data={[
        { label: 'Registers', value: hardwareSpec.numRegisters },
        { label: 'Stacks', value: hardwareSpec.numStacks },
        { label: 'Stack Size', value: hardwareSpec.maxStackLength },
      ]}
    />
  );
};

export default HardwareSpecSummary;
