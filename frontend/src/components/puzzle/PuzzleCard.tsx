import { Card, CardHeader } from "@mui/material";
import { formatCurrency } from "@root/util/format";
import { Puzzle } from "@root/util/types";
import React from "react";
import UnstyledLink from "../common/UnstyledLink";

interface Props {
  puzzle: Puzzle;
}

const PuzzleCard: React.FC<Props> = ({ puzzle }) => (
  <Card
    component={UnstyledLink}
    to={`/puzzles/${puzzle.slug}`}
    sx={{ width: 120 }}
  >
    <CardHeader title={puzzle.name} subheader={formatCurrency(puzzle.reward)} />
  </Card>
);

export default PuzzleCard;
