import React from 'react';
import graphql from 'babel-plugin-relay/macro';
import { ProgramSpecViewQuery } from './__generated__/ProgramSpecViewQuery.graphql';
import { useParams } from 'react-router-dom';
import ProgramSpecDetails from './ProgramSpecDetails';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
}

const ProgramSpecView: React.FC = () => {
  const { hwSlug, programSlug } = useParams<RouteParams>();

  return (
    <QueryResult<ProgramSpecViewQuery>
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
      render={({ props }) => {
        if (props?.hardwareSpec?.programSpec) {
          return (
            <ProgramSpecDetails programSpec={props.hardwareSpec.programSpec} />
          );
        }

        return <NotFoundPage />;
      }}
    />
  );
};

export default ProgramSpecView;
