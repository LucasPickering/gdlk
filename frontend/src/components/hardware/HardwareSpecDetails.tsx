import React from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { HardwareSpecDetails_hardwareSpec } from './__generated__/HardwareSpecDetails_hardwareSpec.graphql';
import {
  Card,
  CardContent,
  Typography,
  Grid,
  CardHeader,
} from '@material-ui/core';
import ProgramSpecListCard from './ProgramSpecListCard';
import HardwareSpecSummary from './HardwareSpecSummary';

const HardwareSpecDetails: React.FC<{
  hardwareSpec: HardwareSpecDetails_hardwareSpec;
  relay: RelayProp;
}> = ({ hardwareSpec }) => {
  return (
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

      <Grid item sm={8} xs={12}>
        <ProgramSpecListCard hardwareSpec={hardwareSpec} />
      </Grid>
    </Grid>
  );
};

export default createFragmentContainer(HardwareSpecDetails, {
  hardwareSpec: graphql`
    fragment HardwareSpecDetails_hardwareSpec on HardwareSpecNode {
      id
      slug
      name
      ...HardwareSpecSummary_hardwareSpec
      ...ProgramSpecListCard_hardwareSpec @arguments(count: 5)
    }
  `,
});
