import React, { useContext } from 'react';
import { useParams } from 'react-router-dom';
import ProgramIde from './ProgramIde';
import NotFoundPage from '@root/components/NotFoundPage';
import PageContainer from '@root/components/common/PageContainer';
import { puzzles } from '@root/data/puzzles';
import { PuzzleSolutionsContext } from '@root/state/user';

interface RouteParams {
  puzzleSlug: string;
  fileName: string;
}

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const { puzzleSlug, fileName } = useParams<RouteParams>();
  const puzzle = puzzles[puzzleSlug];
  const { getPuzzleSolution } = useContext(PuzzleSolutionsContext);
  const puzzleSolution = getPuzzleSolution(puzzleSlug, fileName);

  return (
    <PageContainer
      fullScreen
      navProps={{
        backLink: {
          to: `/puzzles/${puzzleSlug}`,
          label: 'Back to Puzzle',
        },
      }}
    >
      {puzzle && puzzleSolution ? (
        <ProgramIde puzzle={puzzle} puzzleSolution={puzzleSolution} />
      ) : (
        <NotFoundPage />
      )}
    </PageContainer>
  );
};

export default ProgramIdeView;
