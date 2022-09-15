import {
  Button,
  Card,
  CardActionArea,
  CardContent,
  CardHeader,
  Typography,
} from "@mui/material";
import { formatCurrency } from "@root/util/format";
import { Puzzle } from "@root/util/types";
import React from "react";
import UnstyledLink from "../common/UnstyledLink";

interface Props {
  puzzle: Puzzle;
  showDetail?: boolean;
}

/**
 * A card with info on a single puzzle. If specified, show the full description.
 */
const PuzzleCard: React.FC<Props> = ({ puzzle, showDetail = false }) => (
  <Card
    sx={({ transitions }) => ({
      width: showDetail ? 400 : 200,
      transition: transitions.create(["width", "height"]),
    })}
  >
    <CardActionArea component={UnstyledLink} to={`/puzzles/${puzzle.slug}`}>
      <CardHeader
        title={puzzle.name}
        subheader={formatCurrency(puzzle.reward)}
      />

      {showDetail && (
        <CardContent>
          <Typography>{puzzle.description}</Typography>

          <Button
            variant="contained"
            component={UnstyledLink}
            to={`/puzzles/${puzzle.slug}/solution`}
          >
            Edit Solution
          </Button>
        </CardContent>
      )}
    </CardActionArea>
  </Card>
);

export default PuzzleCard;
