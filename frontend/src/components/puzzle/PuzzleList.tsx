import React, { useEffect } from "react";
import {
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  makeStyles,
} from "@material-ui/core";
import { Puzzle } from "@root/util/types";
import UnstyledLink from "../common/UnstyledLink";
import { Done as IconDone } from "@material-ui/icons";
import { useRecoilValue } from "recoil";
import { puzzleCompletionState } from "@root/state/user";

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
    <List dense {...rest}>
      {/* One item per puzzle */}
      {puzzles.map((puzzle) => (
        <PuzzleListItem
          key={puzzle.slug}
          puzzle={puzzle}
          link={link}
          selected={selectedPuzzle === puzzle.slug}
          onClick={onSelectPuzzle && (() => onSelectPuzzle(puzzle.slug))}
        />
      ))}
    </List>
  );
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
      button
      {...(link
        ? {
            component: UnstyledLink,
            to: `/puzzles/${puzzle.slug}`,
          }
        : {})}
      {...rest}
    >
      <ListItemText primary={puzzle.name} />
      {completion === "solved" && (
        <ListItemIcon className={localClasses.solvedIcon}>
          <IconDone />
        </ListItemIcon>
      )}
    </ListItem>
  );
};

export default PuzzleList;
