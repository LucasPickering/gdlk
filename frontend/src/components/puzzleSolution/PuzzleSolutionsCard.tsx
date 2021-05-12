import React from 'react';
import { RelayPaginationProp, createPaginationContainer } from 'react-relay';
import { graphql } from 'react-relay';
import { PuzzleSolutionsCard_puzzle } from './__generated__/PuzzleSolutionsCard_puzzle.graphql';
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
import Link from 'components/common/Link';
import { useRouteMatch } from 'react-router-dom';
import DeletePuzzleSolutionButton from './DeletePuzzleSolutionButton';
import EditPuzzleSolutionButton from './EditPuzzleSolutionButton';
import CreatePuzzleSolutionButton from './CreatePuzzleSolutionButton';
import CopyPuzzleSolutionButton from './CopyPuzzleSolutionButton';

const PuzzleSolutionsCard: React.FC<{
  puzzle: PuzzleSolutionsCard_puzzle;
  relay: RelayPaginationProp;
}> = ({ puzzle }) => {
  const { url } = useRouteMatch();

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
              {puzzle.puzzleSolutions.edges.map(({ node: puzzleSolution }) => (
                <TableRow key={puzzleSolution.id}>
                  <TableCell>
                    <Link to={`${url}/${puzzleSolution.name}`}>
                      {puzzleSolution.name}
                    </Link>
                  </TableCell>

                  <TableCell>
                    <EditPuzzleSolutionButton puzzleSolution={puzzleSolution} />
                    <CopyPuzzleSolutionButton
                      puzzleId={puzzle.id}
                      puzzleSolution={puzzleSolution}
                    />
                    <DeletePuzzleSolutionButton
                      puzzleId={puzzle.id}
                      puzzleSolutionId={puzzleSolution.id}
                      name={puzzleSolution.name}
                    />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </CardContent>

      <CardActions>
        <CreatePuzzleSolutionButton puzzleId={puzzle.id} />
      </CardActions>
    </Card>
  );
};

export default createPaginationContainer(
  PuzzleSolutionsCard,
  {
    puzzle: graphql`
      fragment PuzzleSolutionsCard_puzzle on PuzzleNode
      @argumentDefinitions(
        puzzleSolutionCount: { type: "Int" }
        puzzleSolutionCursor: { type: "String" }
      ) {
        id
        # A user probably won't have a lot of solutions for one program, so
        # don't bother with pagination
        puzzleSolutions(
          first: $puzzleSolutionCount
          after: $puzzleSolutionCursor
        ) @connection(key: "PuzzleSolutionsCard_puzzleSolutions") {
          edges {
            node {
              id
              name
              ...EditPuzzleSolutionButton_puzzleSolution
              ...CopyPuzzleSolutionButton_puzzleSolution
            }
          }
        }
      }
    `,
  },
  {
    direction: 'forward',
    getVariables(props, paginationInfo, fragmentVariables) {
      return {
        ...fragmentVariables,
        ...paginationInfo,
      };
    },
    query: graphql`
      query PuzzleSolutionsCardPaginationQuery($programSlug: String!) {
        puzzle(slug: $programSlug) {
          ...PuzzleSolutionsCard_puzzle
        }
      }
    `,
  }
);
