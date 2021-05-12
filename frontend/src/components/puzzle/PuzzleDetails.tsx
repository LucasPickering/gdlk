import React, { useContext } from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import { graphql } from 'react-relay';
import { PuzzleDetails_puzzle } from './__generated__/PuzzleDetails_puzzle.graphql';
import {
  Card,
  CardContent,
  Typography,
  makeStyles,
  Grid,
} from '@material-ui/core';
import UserProgramsCard from '../puzzleSolution/PuzzleSolutionsCard';
import HardwareSpecSummary from 'components/hardware/HardwareSpecSummary';
import Link from 'components/common/Link';
import { UserContext } from 'state/user';

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
  puzzle: PuzzleDetails_puzzle;
  relay: RelayProp;
}> = ({ puzzle }) => {
  const localClasses = useLocalStyles();
  const { loggedIn } = useContext(UserContext);

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

      {loggedIn && (
        <Grid item sm={6} xs={12}>
          <UserProgramsCard puzzle={puzzle} />
        </Grid>
      )}
    </Grid>
  );
};

export default createFragmentContainer(PuzzleDetails, {
  puzzle: graphql`
    fragment PuzzleDetails_puzzle on PuzzleNode {
      id
      slug
      name
      description
      ...PuzzleSolutionsCard_puzzle
    }
  `,
});
