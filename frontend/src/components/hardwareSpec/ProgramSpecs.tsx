import React from 'react';
import { createPaginationContainer, RelayPaginationProp } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecs_hardwareSpec } from './__generated__/ProgramSpecs_hardwareSpec.graphql';
import { Paper, Button } from '@material-ui/core';

const ProgramSpecs: React.FC<{
  hardwareSpec: ProgramSpecs_hardwareSpec;
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
  ProgramSpecs,
  {
    hardwareSpec: graphql`
      fragment ProgramSpecs_hardwareSpec on HardwareSpecNode
        @argumentDefinitions(
          count: { type: "Int" }
          cursor: { type: "Cursor" }
        ) {
        programSpecs(first: $count, after: $cursor)
          @connection(key: "ProgramSpecs_programSpecs") {
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
      query ProgramSpecsPaginationQuery(
        $hwSlug: String!
        $count: Int
        $cursor: Cursor
      ) {
        hardwareSpec(slug: $hwSlug) {
          ...ProgramSpecs_hardwareSpec
            @arguments(count: $count, cursor: $cursor)
        }
      }
    `,
  }
);
