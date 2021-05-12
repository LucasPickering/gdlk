import React, { useState, useContext } from 'react';
import {
  createPaginationContainer,
  RelayPaginationProp,
  RelayProp,
  createFragmentContainer,
} from 'react-relay';
import { graphql } from 'react-relay';
import {
  Button,
  Card,
  CardContent,
  CardActions,
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
import { useRouteMatch } from 'react-router-dom';
import Link from 'components/common/Link';
import { PuzzleListCard_puzzle } from './__generated__/PuzzleListCard_puzzle.graphql';
import { UserContext } from 'state/user';

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

const PuzzleListRow = createFragmentContainer(
  ({ puzzle }: { puzzle: PuzzleListCard_puzzle; relay: RelayProp }) => {
    const localClasses = useLocalStyles();
    const { url } = useRouteMatch();
    const [open, setOpen] = useState<boolean>(false);

    return (
      <>
        <TableRow className={localClasses.puzzleRow}>
          <TableCell>
            <IconButton onClick={() => setOpen((prev) => !prev)}>
              {open ? <IconExpandLess /> : <IconExpandMore />}
            </IconButton>
          </TableCell>
          <TableCell>
            <Link to={`${url}/puzzles/${puzzle.slug}`}>{puzzle.name}</Link>
          </TableCell>

          <TableCell align="right">
            {puzzle.puzzleSolutions.totalCount}
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
  },
  {
    puzzle: graphql`
      fragment PuzzleListCard_puzzle on PuzzleNode {
        slug
        name
        description
        puzzleSolutions {
          totalCount
        }
      }
    `,
  }
);

const PuzzleListCard: React.FC<{
  query: PuzzleListCard_query;
  relay: RelayPaginationProp;
}> = ({ query, relay: { hasMore, loadMore } }) => {
  const { loggedIn } = useContext(UserContext);
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
                {loggedIn && <TableCell align="right">Solutions</TableCell>}
              </TableRow>
            </TableHead>
            <TableBody>
              {/* One row per program spec*/}
              {query.puzzles.edges.map(({ node: puzzle }) => (
                <PuzzleListRow key={puzzle.id} puzzle={puzzle} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </CardContent>

      {hasMore() && (
        <CardActions>
          <Button size="small" onClick={() => loadMore(5)}>
            Show More
          </Button>
        </CardActions>
      )}
    </Card>
  );
};

export default createPaginationContainer(
  PuzzleListCard,
  {
    query: graphql`
      fragment PuzzleListCard_query on Query
      @argumentDefinitions(count: { type: "Int" }, cursor: { type: "String" }) {
        puzzles(first: $count, after: $cursor)
          @connection(key: "PuzzleList_puzzles") {
          edges {
            node {
              id
              ...PuzzleListCard_puzzle
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
      query PuzzleListCardPaginationQuery($count: Int, $cursor: String) {
        ...PuzzleListCard_query @arguments(count: $count, cursor: $cursor)
      }
    `,
  }
);
