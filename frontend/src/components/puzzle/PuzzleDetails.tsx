import React from 'react';
import { Typography, Grid, Button } from '@material-ui/core';
import { Puzzle } from '@root/util/types';
import UnstyledLink from '../common/UnstyledLink';

const PuzzleDetails: React.FC<{
  puzzle: Puzzle;
}> = ({ puzzle }) => (
  <Grid container>
    <Grid item xs={12}>
      <Typography variant="h1">{puzzle.name}</Typography>
    </Grid>

    <Grid item xs={12}>
      <Typography>{puzzle.description}</Typography>
    </Grid>

    <Grid item xs={12}>
      <Button
        variant="contained"
        component={UnstyledLink}
        to={`/puzzles/${puzzle.slug}/solution`}
      >
        Edit Solution
      </Button>
    </Grid>
  </Grid>
);

export default PuzzleDetails;
