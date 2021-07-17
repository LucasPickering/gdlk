import React from 'react';
import SimpleTable from '@root/components/common/SimpleTable';
import { HardwareSpec } from '@root/util/types';

const HardwareSpecSummary: React.FC<{
  hardwareSpec: HardwareSpec;
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

export default HardwareSpecSummary;
