import React, { useContext } from 'react';
import { FileCopy as IconFileCopy } from '@material-ui/icons';
import IconButton from '@root/components/common/IconButton';
import { PuzzleSolutionsContext } from '@root/state/user';

/**
 * A button that duplicates an existing puzzle solution.
 */
const CopyPuzzleSolutionButton: React.FC<{
  puzzleSlug: string;
  fileName: string;
}> = ({ puzzleSlug, fileName }) => {
  const { copySolution } = useContext(PuzzleSolutionsContext);

  return (
    <IconButton
      aria-label="Copy solution"
      onClick={() => {
        // TODO catch error
        copySolution(puzzleSlug, fileName);
      }}
    >
      <IconFileCopy />
    </IconButton>
  );
};

export default CopyPuzzleSolutionButton;
