import { CircularProgress } from '@material-ui/core';
import React from 'react';
import { QueryRenderer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import environment from 'util/environment';
import ProgramSpecList from './ProgramSpecList';
import { HardwareSpecViewQuery } from './__generated__/HardwareSpecViewQuery.graphql';
import { useParams } from 'react-router-dom';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();

  return (
    <QueryRenderer<HardwareSpecViewQuery>
      environment={environment}
      query={graphql`
        query HardwareSpecViewQuery($hwSlug: String!) {
          hardwareSpec(slug: $hwSlug) {
            id
            ...ProgramSpecList_hardwareSpec @arguments(count: 1)
          }
        }
      `}
      variables={{ hwSlug }}
      render={({ props, error }) => {
        if (error) {
          return <div>error!</div>;
        }

        if (props?.hardwareSpec) {
          return <ProgramSpecList hardwareSpec={props.hardwareSpec} />;
        }

        return <CircularProgress />;
      }}
    />
  );
};

export default HardwareSpecView;
