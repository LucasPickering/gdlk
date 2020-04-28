import { CircularProgress } from '@material-ui/core';
import React from 'react';
import { QueryRenderer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import environment from 'util/environment';
import HardwareSpecList from './HardwareSpecList';
import { HardwareSpecListViewQuery } from './__generated__/HardwareSpecListViewQuery.graphql';
import { useParams } from 'react-router-dom';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecListView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();

  return (
    <QueryRenderer<HardwareSpecListViewQuery>
      environment={environment}
      query={graphql`
        query HardwareSpecListViewQuery {
          ...HardwareSpecList_query @arguments(count: 10)
        }
      `}
      variables={{ hwSlug }}
      render={({ props, error }) => {
        if (error) {
          return <div>error!</div>;
        }

        if (props) {
          return <HardwareSpecList query={props} />;
        }

        return <CircularProgress />;
      }}
    />
  );
};

export default HardwareSpecListView;
