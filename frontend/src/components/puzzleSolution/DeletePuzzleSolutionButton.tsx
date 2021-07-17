import React, { useContext, useState } from 'react';
import { IconButton } from '@material-ui/core';
import { Delete as IconDelete } from '@material-ui/icons';
import ConfirmationDialog from '@root/components/common/ConfirmationDialog';
import { PuzzleSolutionsContext } from '@root/state/user';

/**
 * A button that deletes a puzzle solution (with a confirmation modal).
 */
const DeletePuzzleSolutionButton: React.FC<{
  puzzleSlug: string;
  fileName: string;
}> = ({ puzzleSlug, fileName }) => {
  const [confirmationOpen, setConfirmationOpen] = useState<boolean>(false);
  const { deleteSolution } = useContext(PuzzleSolutionsContext);

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
        confirmText="Delete"
        confirmColor="secondary"
        onConfirm={() => {
          // TODO catch error
          deleteSolution(puzzleSlug, fileName);
        }}
        onClose={() => setConfirmationOpen(false)}
      >
        Are you sure you want to delete {fileName}?
      </ConfirmationDialog>
    </>
  );
};

export default DeletePuzzleSolutionButton;
