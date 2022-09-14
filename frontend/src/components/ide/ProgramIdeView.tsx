import React from "react";
import { useParams } from "react-router-dom";
import ProgramIde from "./ProgramIde";
import NotFoundPage from "@root/components/NotFoundPage";
import { puzzles } from "@root/data/puzzles";

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const { puzzleSlug } = useParams();
  const puzzle = puzzleSlug && puzzles[puzzleSlug];

  if (!puzzle) {
    return <NotFoundPage />;
  }

  return <ProgramIde puzzle={puzzle} />;
};

export default ProgramIdeView;
