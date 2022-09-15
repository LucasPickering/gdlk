import React from "react";
import { puzzles } from "@root/data/puzzles";
import { useParams } from "react-router-dom";
import { Grid } from "@mui/material";
import PuzzleCard from "./PuzzleCard";

const PuzzleListView: React.FC = () => {
  const { puzzleSlug } = useParams();
  return (
    <Grid container spacing={2}>
      {Object.values(puzzles).map((puzzle) => (
        <Grid key={puzzle.slug} item>
          <PuzzleCard puzzle={puzzle} showDetail={puzzleSlug === puzzle.slug} />
        </Grid>
      ))}
    </Grid>
  );
};

export default PuzzleListView;
