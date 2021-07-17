import React from 'react';
import {
  Card,
  CardContent,
  Typography,
  makeStyles,
  Grid,
} from '@material-ui/core';
import PuzzleSolutionsCard from '../puzzleSolution/PuzzleSolutionsCard';
import { Puzzle } from '@root/util/types';

const useLocalStyles = makeStyles(({ spacing }) => ({
  specSection: {
    margin: `${spacing(1)}px 0`,
  },

  programStats: {
    borderCollapse: 'collapse',
  },
  programStatName: {
    textAlign: 'left',
    paddingRight: spacing(2),
  },
  programStatValue: {
    textAlign: 'right',
  },
}));

const PuzzleDetails: React.FC<{
  puzzle: Puzzle;
}> = ({ puzzle }) => {
  const localClasses = useLocalStyles();

  return (
    <Grid container>
      <Grid item xs={12}>
        <Typography variant="h1">{puzzle.name}</Typography>
      </Grid>

      <Grid item sm={6} xs={12}>
        <Card>
          <CardContent>
            <div className={localClasses.specSection}>
              <Typography variant="h2">Description</Typography>
              <Typography>{puzzle.description}</Typography>
            </div>
          </CardContent>
        </Card>
      </Grid>

      <Grid item sm={6} xs={12}>
        <PuzzleSolutionsCard puzzleSlug={puzzle.slug} />
      </Grid>
    </Grid>
  );
};

export default PuzzleDetails;
