import React, { useContext } from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecDetails_programSpec } from './__generated__/ProgramSpecDetails_programSpec.graphql';
import {
  Card,
  CardContent,
  Typography,
  makeStyles,
  Grid,
} from '@material-ui/core';
import UserProgramsCard from '../userPrograms/UserProgramsCard';
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

const ProgramSpecDetails: React.FC<{
  programSpec: ProgramSpecDetails_programSpec;
  relay: RelayProp;
}> = ({ programSpec }) => {
  const localClasses = useLocalStyles();
  const { loggedIn } = useContext(UserContext);

  return (
    <Grid container>
      <Grid item xs={12}>
        <Typography variant="h1">
          <Link to={`/hardware/${programSpec.hardwareSpec.slug}`}>
            {programSpec.hardwareSpec.name}
          </Link>
          {' / '}
          {programSpec.name}
        </Typography>
      </Grid>

      <Grid item sm={6} xs={12}>
        <Card>
          <CardContent>
            <div className={localClasses.specSection}>
              <Typography variant="h2">Description</Typography>
              <Typography>{programSpec.description}</Typography>
            </div>

            <div className={localClasses.specSection}>
              <Typography variant="h2">Hardware Spec</Typography>
              <HardwareSpecSummary hardwareSpec={programSpec.hardwareSpec} />
            </div>
          </CardContent>
        </Card>
      </Grid>

      {loggedIn && (
        <Grid item sm={6} xs={12}>
          <UserProgramsCard programSpec={programSpec} />
        </Grid>
      )}
    </Grid>
  );
};

export default createFragmentContainer(ProgramSpecDetails, {
  programSpec: graphql`
    fragment ProgramSpecDetails_programSpec on ProgramSpecNode {
      id
      slug
      name
      description
      hardwareSpec {
        slug
        name
        ...HardwareSpecSummary_hardwareSpec
      }
      # Requesting user programs while not logged in causes an error
      ...UserProgramsCard_programSpec @include(if: $loggedIn)
    }
  `,
});
