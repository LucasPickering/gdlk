import React from 'react';
import {
  Card,
  CardContent,
  Typography,
  makeStyles,
  CardHeader,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
} from '@material-ui/core';
import { Puzzle } from '@root/util/types';
import UnstyledLink from '../common/UnstyledLink';
import { Done as IconDone } from '@material-ui/icons';
import { useRecoilValue } from 'recoil';
import { puzzleCompletionState } from '@root/state/user';

const useLocalStyles = makeStyles({
  puzzleList: {
    padding: 0,
  },
});

const PuzzleListCard: React.FC<{
  puzzles: Puzzle[];
}> = ({ puzzles }) => {
  const localClasses = useLocalStyles();

  return (
    <Card>
      <CardHeader title={<Typography variant="h2">Puzzles</Typography>} />
      <CardContent>
        <List className={localClasses.puzzleList} dense>
          {/* One item per puzzle */}
          {puzzles.map((puzzle) => (
            <PuzzleListItem key={puzzle.slug} puzzle={puzzle} />
          ))}
        </List>
      </CardContent>
    </Card>
  );
};

const PuzzleListItem: React.FC<{ puzzle: Puzzle }> = ({ puzzle }) => {
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
        <ListItemIcon>
          <IconDone />
        </ListItemIcon>
      )}
    </ListItem>
  );
};

export default PuzzleListCard;
