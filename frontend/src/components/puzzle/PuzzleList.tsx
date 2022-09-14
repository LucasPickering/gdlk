import React, { useEffect } from "react";
import {
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  ListItemButton,
  Grid,
} from "@mui/material";
import { makeStyles } from "@mui/styles";
import { Puzzle } from "@root/util/types";
import UnstyledLink from "../common/UnstyledLink";
import { Done as IconDone } from "@mui/icons-material";
import { useRecoilValue } from "recoil";
import { puzzleCompletionState } from "@root/state/user";
import PuzzleCard from "./PuzzleCard";

const useLocalStyles = makeStyles(({ palette }) => ({
  solvedIcon: {
    color: palette.success.main,
  },
}));

const PuzzleList: React.FC<
  {
    puzzles: Puzzle[];
    link?: boolean;
    selectedPuzzle?: string;
    onSelectPuzzle?: (puzzleSlug: string | undefined) => void;
  } & React.ComponentProps<typeof List>
> = ({ puzzles, link = false, selectedPuzzle, onSelectPuzzle, ...rest }) => {
  // Un-select current puzzle when this component unmounts
  useEffect(
    () => {
      if (onSelectPuzzle) {
        return () => onSelectPuzzle(undefined);
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );

  return (
    <Grid container spacing={2}>
      {puzzles.map((puzzle) => (
        <Grid key={puzzle.slug} item>
          <PuzzleCard puzzle={puzzle} />
        </Grid>
      ))}
    </Grid>
  );
  // <PuzzleListItem
  //   key={puzzle.slug}
  //   puzzle={puzzle}
  //   link={link}
  //   selected={selectedPuzzle === puzzle.slug}
  //   onClick={onSelectPuzzle && (() => onSelectPuzzle(puzzle.slug))}
  // />
};

const PuzzleListItem: React.FC<
  {
    puzzle: Puzzle;
    link?: boolean;
  } & Omit<React.ComponentProps<typeof ListItem>, "button">
> = ({ puzzle, link, ...rest }) => {
  const localClasses = useLocalStyles();
  const completion = useRecoilValue(
    puzzleCompletionState({ puzzleSlug: puzzle.slug })
  );

  return (
    <ListItem
      key={puzzle.name}
      disabled={completion === "locked"}
      {...(link
        ? {
            component: UnstyledLink,
            to: `/puzzles/${puzzle.slug}`,
          }
        : {})}
      {...rest}
    >
      <ListItemButton>
        <ListItemText primary={puzzle.name} />
        {completion === "solved" && (
          <ListItemIcon className={localClasses.solvedIcon}>
            <IconDone />
          </ListItemIcon>
        )}
      </ListItemButton>
    </ListItem>
  );
};

export default PuzzleList;
