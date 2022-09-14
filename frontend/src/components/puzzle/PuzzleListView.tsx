import React from "react";
import { puzzles } from "@root/data/puzzles";
import { Outlet, useParams } from "react-router-dom";
import PuzzleList from "./PuzzleList";

const PuzzleListView: React.FC = () => {
  const { puzzleSlug } = useParams();
  return (
    <>
      <PuzzleList
        puzzles={Object.values(puzzles)}
        selectedPuzzle={puzzleSlug}
        link
      />
      <Outlet />
    </>
  );
};

export default PuzzleListView;
