import React from 'react';
import { useParams } from 'react-router-dom';
import PuzzleDetails from './PuzzleDetails';
import NotFoundPage from '@root/components/NotFoundPage';
import { puzzles } from '@root/data/puzzles';

interface RouteParams {
  puzzleSlug: string;
}

const PuzzleDetailsView: React.FC = () => {
  const { puzzleSlug } = useParams<RouteParams>();
  const puzzle = puzzles[puzzleSlug];

  if (puzzle) {
    return <PuzzleDetails puzzle={puzzle} />;
  }

  return <NotFoundPage />;
};

export default PuzzleDetailsView;
