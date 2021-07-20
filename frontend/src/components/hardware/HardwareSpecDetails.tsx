import React from 'react';
import {
  Card,
  CardContent,
  Typography,
  Grid,
  CardHeader,
} from '@material-ui/core';
import { HardwareSpec } from '@root/util/types';
import SimpleTable from '../common/SimpleTable';

const HardwareSpecDetails: React.FC<{
  hardwareSpec: HardwareSpec;
}> = ({ hardwareSpec }) => (
  <Grid container>
    <Grid item xs={12}>
      <Typography variant="h1">{hardwareSpec.name}</Typography>
    </Grid>

    <Grid item sm={4} xs={12}>
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
    </Grid>
  </Grid>
);

export default HardwareSpecDetails;
