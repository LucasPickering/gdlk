import React, { useState } from 'react';
import { useMutation } from 'relay-hooks';
import graphql from 'babel-plugin-relay/macro';
import { IconButton } from '@material-ui/core';
import { Delete as IconDelete } from '@material-ui/icons';
import { DeleteUserProgramButton_Mutation } from './__generated__/DeleteUserProgramButton_Mutation.graphql';
import ConfirmationDialog from 'components/common/ConfirmationDialog';

const deleteUserProgramMutation = graphql`
  mutation DeleteUserProgramButton_Mutation($id: ID!) {
    deleteUserProgram(input: { id: $id }) {
      deletedId
    }
  }
`;

/**
 * A button that deletes a user program (with a confirmation modal).
 *
 * @param programSpecId The ID of the program that owns this solution
 * @param userProgramId The user program being deleted
 * @param fileName The name of the user program being deleted
 */
const DeleteUserProgramButton: React.FC<{
  programSpecId: string;
  userProgramId: string;
  fileName: string;
}> = ({ programSpecId, userProgramId, fileName }) => {
  const [mutate, { loading }] = useMutation<DeleteUserProgramButton_Mutation>(
    deleteUserProgramMutation
  );
  const [confirmationOpen, setConfirmationOpen] = useState<boolean>(false);

  return (
    <>
      <IconButton
        aria-label="Delete solution"
        onClick={() => setConfirmationOpen(true)}
      >
        <IconDelete />
      </IconButton>
      <ConfirmationDialog
        open={confirmationOpen}
        label="confirm-delete-user-program"
        title="Delete solution?"
        loading={loading}
        confirmText="Delete"
        confirmColor="secondary"
        onConfirm={() => {
          mutate({
            variables: { id: userProgramId },
            configs: [
              {
                type: 'RANGE_DELETE',
                parentID: programSpecId,
                connectionKeys: [
                  {
                    key: 'UserProgramsCard_userPrograms',
                  },
                ],
                pathToConnection: ['programSpec', 'userPrograms'],
                deletedIDFieldName: 'deletedId',
              },
            ],
            // If the delete is successful, this component will unmount,
            // so we don't have to close the modal manually
          });
        }}
        onClose={() => setConfirmationOpen(false)}
      >
        Are you sure you want to delete {fileName}?
      </ConfirmationDialog>
    </>
  );
};

export default DeleteUserProgramButton;
