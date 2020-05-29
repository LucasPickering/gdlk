import React from 'react';
import { createPaginationContainer, RelayPaginationProp } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { HardwareSpecList_query } from './__generated__/HardwareSpecList_query.graphql';
import {
  Button,
  CardContent,
  CardActions,
  List,
  makeStyles,
} from '@material-ui/core';
import HardwareSpecListItem from './HardwareSpecListItem';

const useLocalStyles = makeStyles(() => ({
  hardwareSpecList: {
    // Just rely on the card's padding
    padding: 0,
  },
}));

const HardwareSpecList: React.FC<{
  query: HardwareSpecList_query;
  relay: RelayPaginationProp;
}> = ({ query, relay: { hasMore, loadMore } }) => {
  const localClasses = useLocalStyles();

  return (
    <>
      <CardContent>
        <List className={localClasses.hardwareSpecList} dense>
          {query.hardwareSpecs.edges.map(({ node: hardwareSpec }) => (
            <HardwareSpecListItem
              key={hardwareSpec.id}
              hardwareSpec={hardwareSpec}
            />
          ))}
        </List>
      </CardContent>
      {hasMore() && (
        <CardActions>
          <Button size="small" onClick={() => loadMore(5)}>
            Show More
          </Button>
        </CardActions>
      )}
    </>
  );
};

export default createPaginationContainer(
  HardwareSpecList,
  {
    query: graphql`
      fragment HardwareSpecList_query on Query
        @argumentDefinitions(
          count: { type: "Int" }
          cursor: { type: "Cursor" }
          programSpecCount: { type: "Int" }
        ) {
        hardwareSpecs(first: $count, after: $cursor)
          @connection(key: "HardwareSpecList_hardwareSpecs") {
          edges {
            node {
              id
              ...HardwareSpecListItem_hardwareSpec
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
      query HardwareSpecListPaginationQuery($count: Int, $cursor: Cursor) {
        ...HardwareSpecList_query @arguments(count: $count, cursor: $cursor)
      }
    `,
  }
);
