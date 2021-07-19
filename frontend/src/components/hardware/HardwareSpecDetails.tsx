import React from 'react';
import {
  Card,
  CardContent,
  Typography,
  Grid,
  CardHeader,
} from '@material-ui/core';
import HardwareSpecSummary from './HardwareSpecSummary';
import { HardwareSpec } from '@root/util/types';

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
          <HardwareSpecSummary hardwareSpec={hardwareSpec} />
        </CardContent>
      </Card>
    </Grid>
  </Grid>
);

export default HardwareSpecDetails;
