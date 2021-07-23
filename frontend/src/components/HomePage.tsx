import React from 'react';
import { Grid, Typography } from '@material-ui/core';
import { puzzles } from '@root/data/puzzles';
import NavMenu from './common/NavMenu';
import PuzzleList from './puzzle/PuzzleList';
import { Route, Switch, useParams } from 'react-router-dom';
import PuzzleDetailsView from './puzzle/PuzzleDetailsView';
import HardwareSpecCard from './hardware/HardwareSpecCard';

interface RouteParams {
  puzzleSlug: string;
}

const HomePage: React.FC = () => {
  const { puzzleSlug } = useParams<RouteParams>();

  return (
    <Grid container>
      <Grid item xs={12}>
        <Typography variant="h1">GDLK_OS</Typography>
      </Grid>

      <Grid item md={4} sm={8} xs={12}>
        <NavMenu
          items={[
            {
              id: 'puzzles',
              label: 'Puzzles',
              to: '/puzzles',
              children: (
                <PuzzleList
                  puzzles={Object.values(puzzles)}
                  selectedPuzzle={puzzleSlug}
                  link
                />
              ),
            },
            {
              id: 'hardware',
              label: 'Hardware',
              to: '/hardware',
            },
            {
              id: 'docs',
              label: 'GDLK Reference Guide',
              to: '/docs',
            },
          ]}
        />
      </Grid>

      <Grid item md={8} sm={12}>
        <Switch>
          <Route path="/hardware" exact>
            <HardwareSpecCard />
          </Route>
          <Route path="/puzzles/:puzzleSlug" exact>
            <PuzzleDetailsView />
          </Route>
        </Switch>
      </Grid>
    </Grid>
  );
};

export default HomePage;
