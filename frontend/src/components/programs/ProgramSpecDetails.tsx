import React from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecDetails_programSpec } from './__generated__/ProgramSpecDetails_programSpec.graphql';
import { Card, CardContent, Typography, makeStyles } from '@material-ui/core';
import UserProgramsTable from '../userPrograms/UserProgramsTable';
import HardwareSpecSummary from 'components/hardware/HardwareSpecSummary';
import SimpleTable from 'components/common/SimpleTable';
import Link from 'components/common/Link';

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
  return (
    <Card>
      <CardContent>
        <Typography variant="h5" component="h2">
          {programSpec.slug}
        </Typography>

        <div className={localClasses.specSection}>
          <Typography variant="h6" component="h3">
            Hardware Specs (
            <Link to={`/hardware/${programSpec.hardwareSpec.slug}`}>
              {programSpec.hardwareSpec.slug}
            </Link>
            )
          </Typography>
          <HardwareSpecSummary hardwareSpec={programSpec.hardwareSpec} />
        </div>

        <div className={localClasses.specSection}>
          <Typography variant="h6" component="h3">
            Program Specs
          </Typography>

          <SimpleTable
            data={[
              { label: 'Input', value: programSpec.input.join(' ') },
              {
                label: 'Expected Output',
                value: programSpec.expectedOutput.join(' '),
              },
            ]}
          />
        </div>

        <Typography variant="h6" component="h3">
          Solutions
        </Typography>
        <UserProgramsTable programSpec={programSpec} />
      </CardContent>
    </Card>
  );
};

export default createFragmentContainer(ProgramSpecDetails, {
  programSpec: graphql`
    fragment ProgramSpecDetails_programSpec on ProgramSpecNode {
      slug
      input
      expectedOutput
      hardwareSpec {
        slug
        ...HardwareSpecSummary_hardwareSpec
      }
      ...UserProgramsTable_programSpec
    }
  `,
});
