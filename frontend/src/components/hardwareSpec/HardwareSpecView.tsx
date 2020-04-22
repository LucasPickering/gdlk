import { CircularProgress } from '@material-ui/core';
import React from 'react';
import { QueryRenderer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import PageContainer from 'components/common/PageContainer';
import environment from 'util/environment';
import ProgramSpecs from './ProgramSpecs';
import { HardwareSpecViewQuery } from './__generated__/HardwareSpecViewQuery.graphql';
import { useParams } from 'react-router-dom';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpec: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();

  return (
    <PageContainer>
      <QueryRenderer<HardwareSpecViewQuery>
        environment={environment}
        query={graphql`
          query HardwareSpecViewQuery($hwSlug: String!) {
            hardwareSpec(slug: $hwSlug) {
              id
              ...ProgramSpecs_hardwareSpec @arguments(count: 1)
            }
          }
        `}
        variables={{ hwSlug }}
        render={({ props, error }) => {
          if (error) {
            return <div>error!</div>;
          }

          if (props?.hardwareSpec) {
            return <ProgramSpecs hardwareSpec={props.hardwareSpec} />;
          }

          return <CircularProgress />;
        }}
      />
    </PageContainer>
  );
};

export default HardwareSpec;
