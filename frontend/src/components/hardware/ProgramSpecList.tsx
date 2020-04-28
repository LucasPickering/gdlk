import React from 'react';
import { createPaginationContainer, RelayPaginationProp } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecList_hardwareSpec } from './__generated__/ProgramSpecList_hardwareSpec.graphql';
import { Paper, Button } from '@material-ui/core';

const ProgramSpecList: React.FC<{
  hardwareSpec: ProgramSpecList_hardwareSpec;
  relay: RelayPaginationProp;
}> = ({ hardwareSpec, relay: { hasMore, loadMore } }) => {
  return (
    <div>
      {hardwareSpec.programSpecs.edges.map(({ node: { slug } }) => (
        <Paper key={slug}>{slug}</Paper>
      ))}
      <Button disabled={!hasMore()} onClick={() => loadMore(1)}>
        Load More
      </Button>
    </div>
  );
};

export default createPaginationContainer(
  ProgramSpecList,
  {
    hardwareSpec: graphql`
      fragment ProgramSpecList_hardwareSpec on HardwareSpecNode
        @argumentDefinitions(
          count: { type: "Int" }
          cursor: { type: "Cursor" }
        ) {
        programSpecs(first: $count, after: $cursor)
          @connection(key: "ProgramSpecList_programSpecs") {
          edges {
            node {
              slug
              input
              expectedOutput
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
      query ProgramSpecListPaginationQuery(
        $hwSlug: String!
        $count: Int
        $cursor: Cursor
      ) {
        hardwareSpec(slug: $hwSlug) {
          ...ProgramSpecList_hardwareSpec
            @arguments(count: $count, cursor: $cursor)
        }
      }
    `,
  }
);
