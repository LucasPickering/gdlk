import React from 'react';
import { RelayPaginationProp, createPaginationContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { UserProgramsCard_programSpec } from './__generated__/UserProgramsCard_programSpec.graphql';
import {
  TableBody,
  TableContainer,
  TableRow,
  TableCell,
  Table,
  TableHead,
  Card,
  CardHeader,
  CardContent,
  CardActions,
  Typography,
} from '@material-ui/core';
import Link from 'components/common/Link';
import { useRouteMatch } from 'react-router-dom';
import DeleteUserProgramButton from './DeleteUserProgramButton';
import EditUserProgramButton from './EditUserProgramButton';
import CreateUserProgramButton from './CreateUserProgramButton';
import CopyUserProgramButton from './CopyUserProgramButton';

const UserProgramsCard: React.FC<{
  programSpec: UserProgramsCard_programSpec;
  relay: RelayPaginationProp;
}> = ({ programSpec }) => {
  const { url } = useRouteMatch();

  return (
    <Card>
      <CardHeader title={<Typography variant="h2">Solutions</Typography>} />
      <CardContent>
        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>File Name</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {/* One row per existing solution */}
              {programSpec.userPrograms.edges.map(({ node: userProgram }) => (
                <TableRow key={userProgram.id}>
                  <TableCell>
                    <Link to={`${url}/${userProgram.fileName}`}>
                      {userProgram.fileName}
                    </Link>
                  </TableCell>

                  <TableCell>
                    <EditUserProgramButton userProgram={userProgram} />
                    <CopyUserProgramButton
                      programSpecId={programSpec.id}
                      userProgram={userProgram}
                    />
                    <DeleteUserProgramButton
                      programSpecId={programSpec.id}
                      userProgramId={userProgram.id}
                      fileName={userProgram.fileName}
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </CardContent>

      <CardActions>
        <CreateUserProgramButton programSpecId={programSpec.id} />
      </CardActions>
    </Card>
  );
};

export default createPaginationContainer(
  UserProgramsCard,
  {
    programSpec: graphql`
      fragment UserProgramsCard_programSpec on ProgramSpecNode
        @argumentDefinitions(
          userProgramCount: { type: "Int" }
          userProgramCursor: { type: "Cursor" }
        ) {
        id
        # A user probably won't have a lot of solutions for one program, so
        # don't bother with pagination
        userPrograms(first: $userProgramCount, after: $userProgramCursor)
          @connection(key: "UserProgramsCard_userPrograms") {
          edges {
            node {
              id
              fileName
              ...EditUserProgramButton_userProgram
              ...CopyUserProgramButton_userProgram
            }
          }
        }
      }
    `,
  },
  {
    direction: 'forward',
    getVariables(props, paginationInfo, fragmentVariables) {
      return {
        ...fragmentVariables,
        ...paginationInfo,
      };
    },
    query: graphql`
      query UserProgramsCardPaginationQuery(
        $hardwareSlug: String!
        $programSlug: String!
      ) {
        hardwareSpec(slug: $hardwareSlug) {
          programSpec(slug: $programSlug) {
            ...UserProgramsCard_programSpec
          }
        }
      }
    `,
  }
);
