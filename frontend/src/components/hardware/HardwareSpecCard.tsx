import React from 'react';
import { Typography, Card, CardHeader, CardContent } from '@material-ui/core';
import { hardwareSpecState } from '@root/state/user';
import { useRecoilValue } from 'recoil';
import SimpleTable from '../common/SimpleTable';

/**
 * Show details on the user's current hardware capabilities
 */
const HardwareSpecCard: React.FC = () => {
  const hardwareSpec = useRecoilValue(hardwareSpecState);

  return (
    <Card>
      <CardHeader title={<Typography variant="h2">Hardware</Typography>} />
      <CardContent>
        <SimpleTable
          data={[
            { label: 'Registers', value: hardwareSpec.numRegisters },
            { label: 'Stacks', value: hardwareSpec.numStacks },
            { label: 'Stack Size', value: hardwareSpec.maxStackLength },
          ]}
        />
      </CardContent>
    </Card>
  );
};

export default HardwareSpecCard;
