import React from "react";
import { useNavigate, useParams } from "react-router-dom";
import PuzzleDetails from "./PuzzleDetails";
import NotFoundPage from "@root/components/NotFoundPage";
import { puzzles } from "@root/data/puzzles";
import { Drawer } from "@mui/material";

const PuzzleDetailsView: React.FC = () => {
  const { puzzleSlug } = useParams();
  const puzzle = puzzleSlug && puzzles[puzzleSlug];
  const navigate = useNavigate();

  return (
    <Drawer anchor="right" open onClose={() => navigate("/puzzles")}>
      {puzzle ? <PuzzleDetails puzzle={puzzle} /> : <NotFoundPage />}
    </Drawer>
  );
};

export default PuzzleDetailsView;
