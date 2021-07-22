import React from 'react';
import { Grid, Typography } from '@material-ui/core';
import MainMenu from './navigation/MainMenu';

const HomePage: React.FC = () => (
  <Grid container>
    <Grid item xs={12}>
      <Typography variant="h1">GDLK Development Language Kit</Typography>
    </Grid>

    <Grid item md={4} sm={8} xs={12}>
      <MainMenu />
    </Grid>
  </Grid>
);

export default HomePage;
