import React from 'react';
import HardwareSpecListView from './hardware/HardwareSpecListView';
import { Grid } from '@material-ui/core';

const HomePage: React.FC = () => {
  return (
    <Grid container justify="center">
      <Grid item md={4} sm={8} xs={12}>
        <HardwareSpecListView />
      </Grid>
    </Grid>
  );
};

export default HomePage;
