import React, { useContext } from 'react';
import {
  TableBody,
  TableContainer,
  TableRow,
  TableCell,
  Table,
  TableHead,
  Card,
  CardHeader,
  CardContent,
  CardActions,
  Typography,
} from '@material-ui/core';
import Link from '@root/components/common/Link';
import { useRouteMatch } from 'react-router-dom';
import DeletePuzzleSolutionButton from './DeletePuzzleSolutionButton';
import EditPuzzleSolutionButton from './EditPuzzleSolutionButton';
import CreatePuzzleSolutionButton from './CreatePuzzleSolutionButton';
import CopyPuzzleSolutionButton from './CopyPuzzleSolutionButton';
import { PuzzleSolutionsContext } from '@root/state/user';

const PuzzleSolutionsCard: React.FC<{
  puzzleSlug: string;
}> = ({ puzzleSlug }) => {
  const { url } = useRouteMatch();
  const { getPuzzleSolutions } = useContext(PuzzleSolutionsContext);
  const puzzleSolutions = getPuzzleSolutions(puzzleSlug);

  return (
    <Card>
      <CardHeader title={<Typography variant="h2">Solutions</Typography>} />
      <CardContent>
        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>File Name</TableCell>
                <TableCell>Actions</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {/* One row per existing solution */}
              {puzzleSolutions.map((puzzleSolution) => (
                <TableRow key={puzzleSolution.fileName}>
                  <TableCell>
                    <Link to={`${url}/${puzzleSolution.fileName}`}>
                      {puzzleSolution.fileName}
                    </Link>
                  </TableCell>

                  <TableCell>
                    <EditPuzzleSolutionButton
                      puzzleSlug={puzzleSlug}
                      fileName={puzzleSolution.fileName}
                    />
                    <CopyPuzzleSolutionButton
                      puzzleSlug={puzzleSlug}
                      fileName={puzzleSolution.fileName}
                    />
                    <DeletePuzzleSolutionButton
                      puzzleSlug={puzzleSlug}
                      fileName={puzzleSolution.fileName}
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </CardContent>

      <CardActions>
        <CreatePuzzleSolutionButton puzzleSlug={puzzleSlug} />
      </CardActions>
    </Card>
  );
};

export default PuzzleSolutionsCard;
