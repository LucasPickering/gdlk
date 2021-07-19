import React, { useContext, useState } from 'react';
import {
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
} from '@material-ui/core';
import { Edit as IconEdit } from '@material-ui/icons';
import EditPuzzleSolutionForm from './EditPuzzleSolutionForm';
import { PuzzleSolutionsContext } from '@root/state/user';

/**
 * A button to open a modal that allows the user to edit the METADATA for a
 * puzzle solution (NOT the source code).
 */
const EditPuzzleSolutionButton: React.FC<{
  puzzleSlug: string;
  fileName: string;
}> = ({ puzzleSlug, fileName }) => {
  const { renameSolution } = useContext(PuzzleSolutionsContext);
  const [modalOpen, setModalOpen] = useState<boolean>(false);

  return (
    <>
      <IconButton aria-label="Edit solution" onClick={() => setModalOpen(true)}>
        <IconEdit />
      </IconButton>

      <Dialog
        aria-labelledby="edit-user-program-dialog-title"
        open={modalOpen}
        onClose={() => setModalOpen(false)}
      >
        <DialogTitle id="edit-user-program-dialog-title">
          Rename solution
        </DialogTitle>
        <DialogContent>
          <EditPuzzleSolutionForm
            fileName={fileName}
            onSubmit={({ fileName: newFileName }) => {
              // TODO catch error
              renameSolution(puzzleSlug, fileName, newFileName);
              setModalOpen(false);
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default EditPuzzleSolutionButton;
