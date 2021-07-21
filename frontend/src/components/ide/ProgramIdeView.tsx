import React from 'react';
import { useParams } from 'react-router-dom';
import ProgramIde from './ProgramIde';
import NotFoundPage from '@root/components/NotFoundPage';
import PageContainer from '@root/components/common/PageContainer';
import { puzzles } from '@root/data/puzzles';

interface RouteParams {
  puzzleSlug: string;
}

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const { puzzleSlug } = useParams<RouteParams>();
  const puzzle = puzzles[puzzleSlug];

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
      {puzzle ? <ProgramIde puzzle={puzzle} /> : <NotFoundPage />}
    </PageContainer>
  );
};

export default ProgramIdeView;
