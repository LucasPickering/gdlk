import React, { useState } from 'react';
import { Dialog, DialogTitle, DialogContent, Button } from '@material-ui/core';
import { Add as IconAdd } from '@material-ui/icons';
import EditUserProgramForm from './EditUserProgramForm';
import graphql from 'babel-plugin-relay/macro';
import { useMutation } from 'relay-hooks';
import { CreateUserProgramButton_Mutation } from './__generated__/CreateUserProgramButton_Mutation.graphql';

const createUserProgramMutation = graphql`
  mutation CreateUserProgramButton_Mutation(
    $programSpecId: ID!
    $fileName: String!
  ) {
    createUserProgram(
      input: { programSpecId: $programSpecId, fileName: $fileName }
    ) {
      userProgramEdge {
        node {
          fileName
        }
      }
    }
  }
`;

/**
 * A button to open a modal that creates a new a user program.
 *
 * @param className Optional CSS class name
 * @param programSpecId The ID of the program that will own the new user program
 */
const CreateUserProgramButton: React.FC<{
  className?: string;
  programSpecId: string;
}> = ({ className, programSpecId }) => {
  const [mutate, { loading }] = useMutation<CreateUserProgramButton_Mutation>(
    createUserProgramMutation
  );
  const [modalOpen, setModalOpen] = useState<boolean>(false);

  return (
    <>
      <Button
        className={className}
        startIcon={<IconAdd />}
        size="small"
        onClick={() => setModalOpen(true)}
      >
        New Solution
      </Button>
      <Dialog
        aria-labelledby="new-user-program-dialog-title"
        open={modalOpen}
        onClose={() => setModalOpen(false)}
      >
        <DialogTitle id="new-user-program-dialog-title">
          Create new solution
        </DialogTitle>
        <DialogContent>
          <EditUserProgramForm
            loading={loading}
            onSubmit={({ fileName }) => {
              mutate({
                variables: { programSpecId, fileName },
                configs: [
                  // Update the list of programs in the parent after modification
                  {
                    type: 'RANGE_ADD',
                    parentID: programSpecId,
                    connectionInfo: [
                      {
                        key: 'UserProgramsCard_userPrograms',
                        rangeBehavior: 'append',
                      },
                    ],
                    edgeName: 'userProgramEdge',
                  },
                ],
                onCompleted: () => setModalOpen(false),
              });
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default CreateUserProgramButton;
