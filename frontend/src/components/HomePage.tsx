import React from 'react';
import { Grid, Typography } from '@material-ui/core';
import { puzzles } from '@root/data/puzzles';
import NavMenu from './common/NavMenu';
import PuzzleList from './puzzle/PuzzleList';
import PuzzleDetails from './puzzle/PuzzleDetails';
import { useParams } from 'react-router-dom';

interface RouteParams {
  puzzleSlug: string;
}

const HomePage: React.FC = () => {
  const { puzzleSlug } = useParams<RouteParams>();
  const puzzle = puzzleSlug ? puzzles[puzzleSlug] : undefined;

  return (
    <Grid container>
      <Grid item xs={12}>
        <Typography variant="h1">GDLK Development Language Kit</Typography>
      </Grid>

      <Grid item md={4} sm={8} xs={12}>
        <NavMenu
          items={[
            {
              id: 'puzzles',
              label: 'Puzzles',
              children: (
                <PuzzleList
                  puzzles={Object.values(puzzles)}
                  selectedPuzzle={puzzleSlug}
                  link
                />
              ),
            },
            { id: 'docs', label: 'GDLK Language Reference', to: '/docs' },
          ]}
          // If a puzzle is defined in the route, pre-expand the puzzle option
          initialExpandedItem={puzzle && 'puzzles'}
        />
      </Grid>
      {puzzle && (
        <Grid item md={8} sm={12}>
          <PuzzleDetails puzzle={puzzle} />
        </Grid>
      )}
    </Grid>
  );
};

export default HomePage;
