import React, { useState } from 'react';
import { useMutation } from 'relay-hooks';
import { graphql } from 'react-relay';
import { IconButton } from '@material-ui/core';
import { Delete as IconDelete } from '@material-ui/icons';
import { DeletePuzzleSolutionButton_Mutation } from './__generated__/DeletePuzzleSolutionButton_Mutation.graphql';
import ConfirmationDialog from 'components/common/ConfirmationDialog';

const deletePuzzleSolutionMutation = graphql`
  mutation DeletePuzzleSolutionButton_Mutation($id: ID!) {
    deletePuzzleSolution(input: { id: $id }) {
      puzzleSolution {
        id
      }
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
const DeletePuzzleSolutionButton: React.FC<{
  programSpecId: string;
  userProgramId: string;
  fileName: string;
}> = ({ programSpecId, userProgramId, fileName }) => {
  const [mutate, { loading }] =
    useMutation<DeletePuzzleSolutionButton_Mutation>(
      deletePuzzleSolutionMutation
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
                    key: 'PuzzleSolutionsCard_userPrograms',
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

export default DeletePuzzleSolutionButton;
