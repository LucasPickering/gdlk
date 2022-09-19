import {
  Button,
  Card,
  CardActionArea,
  CardHeader,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  Typography,
} from "@mui/material";
import { formatCurrency } from "@root/util/format";
import { Puzzle } from "@root/util/types";
import React, { useState } from "react";
import UnstyledLink from "../common/UnstyledLink";

interface Props {
  puzzle: Puzzle;
}

/**
 * A card with info on a single puzzle. If specified, show the full description
 * in a separate modal.
 *
 * TODO the detail view is kinda janky, break it apart once we know what we
 * want it to look like (and probably make it route-enabled).
 */
const PuzzleCard: React.FC<Props> = ({ puzzle }) => {
  const [showDetail, setShowDetail] = useState(false);
  return (
    <>
      <Card sx={{ width: 200 }}>
        <CardActionArea onClick={() => setShowDetail(true)}>
          <CardHeader
            title={puzzle.name}
            subheader={formatCurrency(puzzle.reward)}
          />
        </CardActionArea>
      </Card>

      <Dialog open={showDetail} onClose={() => setShowDetail(false)}>
        <DialogTitle>{puzzle.name}</DialogTitle>

        <DialogContent>
          <Typography component="span" variant="h5">
            {formatCurrency(puzzle.reward)} recoverable
          </Typography>
          <DialogContentText>{puzzle.description}</DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button
            variant="contained"
            component={UnstyledLink}
            to={`/puzzles/${puzzle.slug}/solution`}
          >
            Edit Solution
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};

export default PuzzleCard;
