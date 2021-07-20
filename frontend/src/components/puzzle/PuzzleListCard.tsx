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
} from '@material-ui/core';
import { Puzzle } from '@root/util/types';
import UnstyledLink from '../common/UnstyledLink';

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
            <ListItem
              key={puzzle.name}
              button
              component={UnstyledLink}
              to={`/puzzles/${puzzle.slug}`}
            >
              <ListItemText primary={puzzle.name} />
            </ListItem>
          ))}
        </List>
      </CardContent>
    </Card>
  );
};

export default PuzzleListCard;
