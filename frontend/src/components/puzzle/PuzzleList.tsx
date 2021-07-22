import React from 'react';
import {
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  makeStyles,
} from '@material-ui/core';
import { Puzzle } from '@root/util/types';
import UnstyledLink from '../common/UnstyledLink';
import { Done as IconDone } from '@material-ui/icons';
import { useRecoilValue } from 'recoil';
import { puzzleCompletionState } from '@root/state/user';

const useLocalStyles = makeStyles(({ palette }) => ({
  solvedIcon: {
    color: palette.success.main,
  },
}));

const PuzzleList: React.FC<
  {
    puzzles: Puzzle[];
  } & React.ComponentProps<typeof List>
> = ({ puzzles, ...rest }) => (
  <List dense {...rest}>
    {/* One item per puzzle */}
    {puzzles.map((puzzle) => (
      <PuzzleListItem key={puzzle.slug} puzzle={puzzle} />
    ))}
  </List>
);

const PuzzleListItem: React.FC<{ puzzle: Puzzle }> = ({ puzzle }) => {
  const localClasses = useLocalStyles();
  const completion = useRecoilValue(
    puzzleCompletionState({ puzzleSlug: puzzle.slug })
  );

  return (
    <ListItem
      key={puzzle.name}
      disabled={completion === 'locked'}
      button
      component={UnstyledLink}
      to={`/puzzles/${puzzle.slug}`}
    >
      <ListItemText primary={puzzle.name} />
      {completion === 'solved' && (
        <ListItemIcon className={localClasses.solvedIcon}>
          <IconDone />
        </ListItemIcon>
      )}
    </ListItem>
  );
};

export default PuzzleList;
