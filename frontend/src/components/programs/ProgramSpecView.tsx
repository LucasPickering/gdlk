import { CircularProgress } from '@material-ui/core';
import React from 'react';
import { QueryRenderer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import environment from 'util/environment';
import { ProgramSpecViewQuery } from './__generated__/ProgramSpecViewQuery.graphql';
import { useParams } from 'react-router-dom';
import ProgramSpecDetails from './ProgramSpecDetails';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
}

const ProgramSpecView: React.FC = () => {
  const { hwSlug, programSlug } = useParams<RouteParams>();

  return (
    <QueryRenderer<ProgramSpecViewQuery>
      environment={environment}
      query={graphql`
        query ProgramSpecViewQuery($hwSlug: String!, $programSlug: String!) {
          hardwareSpec(slug: $hwSlug) {
            slug
            numRegisters
            numStacks
            maxStackLength
            programSpec(slug: $programSlug) {
              ...ProgramSpecDetails_programSpec
            }
          }
        }
      `}
      variables={{ hwSlug, programSlug }}
      render={({ props, error }) => {
        if (error) {
          return <div>error!</div>;
        }

        if (props?.hardwareSpec?.programSpec) {
          return (
            <ProgramSpecDetails programSpec={props.hardwareSpec.programSpec} />
          );
        }

        return <CircularProgress />;
      }}
    />
  );
};

export default ProgramSpecView;
