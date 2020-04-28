import React, { useState } from 'react';
import { RelayPaginationProp, createPaginationContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { Add as IconAdd } from '@material-ui/icons';
import { UserProgramsTable_programSpec } from './__generated__/UserProgramsTable_programSpec.graphql';
import {
  makeStyles,
  Typography,
  Dialog,
  DialogTitle,
  DialogContent,
  TableBody,
  TableContainer,
  TableRow,
  TableCell,
  Table,
  TableHead,
  Button,
} from '@material-ui/core';
import Link from 'components/common/Link';
import { useRouteMatch } from 'react-router-dom';
import EditUserProgramForm from './EditUserProgramForm';
import DeleteUserProgramButton from './DeleteUserProgramButton';

const useLocalStyles = makeStyles(({ spacing }) => ({
  newSolutionButton: {
    marginTop: spacing(1),
  },
}));

const UserProgramsTable: React.FC<{
  programSpec: UserProgramsTable_programSpec;
  relay: RelayPaginationProp;
}> = ({ programSpec }) => {
  const localClasses = useLocalStyles();
  const { url } = useRouteMatch();
  const [newSolutionModalOpen, setNewSolutionModalOpen] = useState<boolean>(
    false
  );

  return (
    <>
      <TableContainer>
        <Typography variant="h6" component="h3">
          Solutions
        </Typography>

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
              <TableRow key={userProgram.fileName}>
                <TableCell>
                  <Link to={`${url}/${userProgram.fileName}`}>
                    {userProgram.fileName}
                  </Link>
                </TableCell>

                <TableCell>
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

        <Button
          className={localClasses.newSolutionButton}
          startIcon={<IconAdd />}
          size="small"
          onClick={() => setNewSolutionModalOpen(true)}
        >
          New Solution
        </Button>
      </TableContainer>

      <Dialog
        aria-labelledby="edit-user-program-dialog-title"
        open={newSolutionModalOpen}
        onClose={() => setNewSolutionModalOpen(false)}
      >
        <DialogTitle id="edit-user-program-dialog-title">
          Create new solution
        </DialogTitle>
        <DialogContent>
          <EditUserProgramForm
            programSpecId={programSpec.id}
            onCompleted={() => setNewSolutionModalOpen(false)}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default createPaginationContainer(
  UserProgramsTable,
  {
    programSpec: graphql`
      fragment UserProgramsTable_programSpec on ProgramSpecNode
        @argumentDefinitions(
          userProgramCount: { type: "Int" }
          userProgramCursor: { type: "Cursor" }
        ) {
        id
        # A user probably won't have a lot of solutions for one program, so
        # don't bother with pagination
        userPrograms(first: $userProgramCount, after: $userProgramCursor)
          @connection(key: "UserProgramsTable_userPrograms") {
          edges {
            node {
              id
              fileName
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
      query UserProgramsTablePaginationQuery(
        $hardwareSlug: String!
        $programSlug: String!
      ) {
        hardwareSpec(slug: $hardwareSlug) {
          programSpec(slug: $programSlug) {
            ...UserProgramsTable_programSpec
          }
        }
      }
    `,
  }
);
