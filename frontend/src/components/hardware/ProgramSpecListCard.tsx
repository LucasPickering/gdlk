import React, { useState, useContext } from 'react';
import {
  createPaginationContainer,
  RelayPaginationProp,
  RelayProp,
  createFragmentContainer,
} from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecListCard_hardwareSpec } from './__generated__/ProgramSpecListCard_hardwareSpec.graphql';
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
import { ProgramSpecListCard_programSpec } from './__generated__/ProgramSpecListCard_programSpec.graphql';
import { UserContext } from 'state/user';

const useLocalStyles = makeStyles(({ spacing }) => ({
  programSpecRow: {
    '& > *': {
      borderBottom: 'none',
    },
  },
  programSpecRowExtra: {
    paddingTop: 0,
    paddingBottom: 0,
  },
  programSpecSummaryWrapper: {
    margin: spacing(1),
  },
}));

const ProgramSpecListRow = createFragmentContainer(
  ({
    programSpec,
  }: {
    programSpec: ProgramSpecListCard_programSpec;
    relay: RelayProp;
  }) => {
    const localClasses = useLocalStyles();
    const { url } = useRouteMatch();
    const [open, setOpen] = useState<boolean>(false);

    return (
      <>
        <TableRow className={localClasses.programSpecRow}>
          <TableCell>
            <IconButton onClick={() => setOpen((prev) => !prev)}>
              {open ? <IconExpandLess /> : <IconExpandMore />}
            </IconButton>
          </TableCell>
          <TableCell>
            <Link to={`${url}/puzzles/${programSpec.slug}`}>
              {programSpec.slug}
            </Link>
          </TableCell>

          {programSpec.userPrograms && (
            <TableCell align="right">
              {programSpec.userPrograms.totalCount}
            </TableCell>
          )}
        </TableRow>
        <TableRow>
          <TableCell className={localClasses.programSpecRowExtra} colSpan={3}>
            <Collapse in={open} unmountOnExit>
              <div className={localClasses.programSpecSummaryWrapper}>
                {programSpec.description}
              </div>
            </Collapse>
          </TableCell>
        </TableRow>
      </>
    );
  },
  {
    programSpec: graphql`
      fragment ProgramSpecListCard_programSpec on ProgramSpecNode {
        slug
        description
        userPrograms @include(if: $loggedIn) {
          totalCount
        }
      }
    `,
  }
);

const ProgramSpecListCard: React.FC<{
  hardwareSpec: ProgramSpecListCard_hardwareSpec;
  relay: RelayPaginationProp;
}> = ({ hardwareSpec, relay: { hasMore, loadMore } }) => {
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
              {hardwareSpec.programSpecs.edges.map(({ node: programSpec }) => (
                <ProgramSpecListRow
                  key={programSpec.id}
                  programSpec={programSpec}
                />
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
  ProgramSpecListCard,
  {
    hardwareSpec: graphql`
      fragment ProgramSpecListCard_hardwareSpec on HardwareSpecNode
        @argumentDefinitions(
          count: { type: "Int" }
          cursor: { type: "Cursor" }
        ) {
        programSpecs(first: $count, after: $cursor)
          @connection(key: "ProgramSpecList_programSpecs") {
          edges {
            node {
              id
              ...ProgramSpecListCard_programSpec
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
      query ProgramSpecListCardPaginationQuery(
        $loggedIn: Boolean!
        $hwSlug: String!
        $count: Int
        $cursor: Cursor
      ) {
        hardwareSpec(slug: $hwSlug) {
          ...ProgramSpecListCard_hardwareSpec
            @arguments(count: $count, cursor: $cursor)
        }
      }
    `,
  }
);
