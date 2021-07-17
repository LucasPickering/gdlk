import React, { useContext, useState } from 'react';
import { Dialog, DialogTitle, DialogContent, Button } from '@material-ui/core';
import { Add as IconAdd } from '@material-ui/icons';
import EditPuzzleSolutionForm from './EditPuzzleSolutionForm';
import { PuzzleSolutionsContext } from '@root/state/user';

/**
 * A button to open a modal that creates a new puzzle solution.
 */
const CreatePuzzleSolutionButton: React.FC<{
  className?: string;
  puzzleSlug: string;
}> = ({ className, puzzleSlug }) => {
  const { createSolution } = useContext(PuzzleSolutionsContext);
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
          <EditPuzzleSolutionForm
            onSubmit={({ fileName }) => {
              // TODO catch error
              createSolution(puzzleSlug, fileName);
              setModalOpen(false);
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default CreatePuzzleSolutionButton;
