import React from "react";
import { useParams } from "react-router-dom";
import PuzzleDetails from "./PuzzleDetails";
import NotFoundPage from "@root/components/NotFoundPage";
import { puzzles } from "@root/data/puzzles";

const PuzzleDetailsView: React.FC = () => {
  const { puzzleSlug } = useParams();
  const puzzle = puzzleSlug && puzzles[puzzleSlug];

  if (puzzle) {
    return <PuzzleDetails puzzle={puzzle} />;
  }

  return <NotFoundPage />;
};

export default PuzzleDetailsView;
