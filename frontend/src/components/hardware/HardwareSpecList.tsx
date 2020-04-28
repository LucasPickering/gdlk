import React from 'react';
import { createPaginationContainer, RelayPaginationProp } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { HardwareSpecList_query } from './__generated__/HardwareSpecList_query.graphql';
import { Button } from '@material-ui/core';
import HardwareSpecPanel from './HardwareSpecPanel';

const HardwareSpecList: React.FC<{
  query: HardwareSpecList_query;
  relay: RelayPaginationProp;
}> = ({ query, relay: { hasMore, loadMore } }) => {
  return (
    <div>
      {query.hardwareSpecs.edges.map(({ node: hardwareSpec }) => (
        <HardwareSpecPanel
          key={hardwareSpec.slug}
          hardwareSpec={hardwareSpec}
        />
      ))}
      {hasMore() && <Button onClick={() => loadMore(1)}>Load More</Button>}
    </div>
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
              slug
              ...HardwareSpecPanel_hardwareSpec
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
