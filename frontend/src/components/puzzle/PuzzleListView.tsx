import React from "react";
import { puzzles } from "@root/data/puzzles";
import { Grid } from "@mui/material";
import PuzzleCard from "./PuzzleCard";

const PuzzleListView: React.FC = () => (
  <Grid container spacing={2}>
    {Object.values(puzzles).map((puzzle) => (
      <Grid key={puzzle.slug} item>
        <PuzzleCard puzzle={puzzle} />
      </Grid>
    ))}
  </Grid>
);

export default PuzzleListView;
