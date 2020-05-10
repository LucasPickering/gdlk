import React from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { HardwareSpecSummary_hardwareSpec } from './__generated__/HardwareSpecSummary_hardwareSpec.graphql';
import SimpleTable from 'components/common/SimpleTable';

const HardwareSpecSummary: React.FC<{
  hardwareSpec: HardwareSpecSummary_hardwareSpec;
  relay: RelayProp;
}> = ({ hardwareSpec }) => {
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

export default createFragmentContainer(HardwareSpecSummary, {
  hardwareSpec: graphql`
    fragment HardwareSpecSummary_hardwareSpec on HardwareSpecNode {
      numRegisters
      numStacks
      maxStackLength
    }
  `,
});
