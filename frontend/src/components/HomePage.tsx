import React from 'react';
import { Grid, Typography } from '@material-ui/core';
import { puzzles } from '@root/data/puzzles';
import NavMenu from './common/NavMenu';
import PuzzleList from './puzzle/PuzzleList';

const HomePage: React.FC = () => (
  <Grid container>
    <Grid item xs={12}>
      <Typography variant="h1">GDLK Development Language Kit</Typography>
    </Grid>

    <Grid item md={4} sm={8} xs={12}>
      <NavMenu
        items={[
          {
            label: 'Puzzles',
            children: <PuzzleList puzzles={Object.values(puzzles)} />,
          },
          { label: 'GDLK Language Reference', to: '/docs' },
        ]}
      />
    </Grid>
  </Grid>
);

export default HomePage;
