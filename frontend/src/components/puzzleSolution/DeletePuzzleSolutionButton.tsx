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
 * A button that deletes a puzzle solution (with a confirmation modal).
 *
 * @param puzzleId The ID of the puzzle that owns this solution
 * @param puzzleSolutionId The puzzle solution being deleted
 * @param name The name of the puzzle solution being deleted
 */
const DeletePuzzleSolutionButton: React.FC<{
  puzzleId: string;
  puzzleSolutionId: string;
  name: string;
}> = ({ puzzleId, puzzleSolutionId, name }) => {
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
        label="confirm-delete-puzzle-solution"
        title="Delete solution?"
        loading={loading}
        confirmText="Delete"
        confirmColor="secondary"
        onConfirm={() => {
          mutate({
            variables: { id: puzzleSolutionId },
            configs: [
              {
                type: 'RANGE_DELETE',
                parentID: puzzleId,
                connectionKeys: [
                  {
                    key: 'PuzzleSolutionsCard_puzzleSolutions',
                  },
                ],
                pathToConnection: ['puzzle', 'puzzleSolutions'],
                deletedIDFieldName: 'deletedId',
              },
            ],
            // If the delete is successful, this component will unmount,
            // so we don't have to close the modal manually
          });
        }}
        onClose={() => setConfirmationOpen(false)}
      >
        Are you sure you want to delete {name}?
      </ConfirmationDialog>
    </>
  );
};

export default DeletePuzzleSolutionButton;
