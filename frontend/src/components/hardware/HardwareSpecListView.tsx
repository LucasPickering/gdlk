import React from 'react';
import graphql from 'babel-plugin-relay/macro';
import HardwareSpecList from './HardwareSpecList';
import { HardwareSpecListViewQuery } from './__generated__/HardwareSpecListViewQuery.graphql';
import { useParams } from 'react-router-dom';
import QueryResult from 'components/common/QueryResult';
import { Card, CardHeader, Typography } from '@material-ui/core';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecListView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();

  return (
    <Card>
      <CardHeader
        title={<Typography variant="h2">Hardware Specs</Typography>}
      />
      <QueryResult<HardwareSpecListViewQuery>
        query={graphql`
          query HardwareSpecListViewQuery {
            ...HardwareSpecList_query @arguments(count: 5)
          }
        `}
        variables={{ hwSlug }}
        render={({ props }) => {
          if (props) {
            return <HardwareSpecList query={props} />;
          }

          // This _shouldn't_ ever happen, since the query result is always populated
          return null;
        }}
      />
    </Card>
  );
};

export default HardwareSpecListView;
