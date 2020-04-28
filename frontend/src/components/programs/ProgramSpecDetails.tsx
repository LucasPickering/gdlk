import React from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecDetails_programSpec } from './__generated__/ProgramSpecDetails_programSpec.graphql';
import { Card, CardContent, Typography } from '@material-ui/core';
import UserProgramsTable from './UserProgramsTable';

const ProgramSpecDetails: React.FC<{
  programSpec: ProgramSpecDetails_programSpec;
  relay: RelayProp;
}> = ({ programSpec }) => {
  return (
    <Card>
      <CardContent>
        <Typography variant="h5" component="h2">
          {programSpec.slug}
        </Typography>
        <Typography>Input: {JSON.stringify(programSpec.input)}</Typography>
        <Typography>
          Expected Output: {JSON.stringify(programSpec.expectedOutput)}
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
      ...UserProgramsTable_programSpec
    }
  `,
});
