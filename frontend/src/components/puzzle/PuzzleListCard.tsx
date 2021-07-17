import React, { useContext, useState } from 'react';
import {
  Card,
  CardContent,
  Typography,
  TableContainer,
  Table,
  TableHead,
  TableRow,
  TableCell,
  TableBody,
  Collapse,
  IconButton,
  makeStyles,
  CardHeader,
} from '@material-ui/core';
import {
  ExpandMore as IconExpandMore,
  ExpandLess as IconExpandLess,
} from '@material-ui/icons';
import Link from '@root/components/common/Link';
import { Puzzle } from '@root/util/types';
import { PuzzleSolutionsContext } from '@root/state/user';

const useLocalStyles = makeStyles(({ spacing }) => ({
  puzzleRow: {
    '& > *': {
      borderBottom: 'none',
    },
  },
  puzzleRowExtra: {
    paddingTop: 0,
    paddingBottom: 0,
  },
  puzzleSummaryWrapper: {
    margin: spacing(1),
  },
}));

const PuzzleListRow: React.FC<{ puzzle: Puzzle }> = ({ puzzle }) => {
  const localClasses = useLocalStyles();
  const [open, setOpen] = useState<boolean>(false);
  const { getPuzzleSolutions } = useContext(PuzzleSolutionsContext);

  return (
    <>
      <TableRow className={localClasses.puzzleRow}>
        <TableCell>
          <IconButton onClick={() => setOpen((prev) => !prev)}>
            {open ? <IconExpandLess /> : <IconExpandMore />}
          </IconButton>
        </TableCell>
        <TableCell>
          <Link to={`/puzzles/${puzzle.slug}`}>{puzzle.name}</Link>
        </TableCell>

        <TableCell align="right">
          {getPuzzleSolutions(puzzle.slug).length || '–'}
        </TableCell>
      </TableRow>
      <TableRow>
        <TableCell className={localClasses.puzzleRowExtra} colSpan={3}>
          <Collapse in={open} unmountOnExit>
            <div className={localClasses.puzzleSummaryWrapper}>
              {puzzle.description}
            </div>
          </Collapse>
        </TableCell>
      </TableRow>
    </>
  );
};

const PuzzleListCard: React.FC<{
  puzzles: Puzzle[];
}> = ({ puzzles }) => {
  return (
    <Card>
      <CardHeader title={<Typography variant="h2">Puzzles</Typography>} />
      <CardContent>
        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell />
                <TableCell>Puzzle</TableCell>
                <TableCell align="right">Solutions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {/* One row per puzzle */}
              {puzzles.map((puzzle) => (
                <PuzzleListRow key={puzzle.slug} puzzle={puzzle} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </CardContent>
    </Card>
  );
};

export default PuzzleListCard;
