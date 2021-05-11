import React, { useContext } from 'react';
import { graphql } from 'react-relay';
import { PuzzleViewQuery } from './__generated__/PuzzleViewQuery.graphql';
import { useParams } from 'react-router-dom';
import PuzzleDetails from './PuzzleDetails';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import { UserContext } from 'state/user';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
}

const PuzzleView: React.FC = () => {
  const { hwSlug, programSlug } = useParams<RouteParams>();
  const { loggedIn } = useContext(UserContext);

  return (
    <QueryResult<PuzzleViewQuery>
      query={graphql`
        query PuzzleViewQuery(
          $loggedIn: Boolean!
          $hwSlug: String!
          $programSlug: String!
        ) {
          hardwareSpec(slug: $hwSlug) {
            slug
            numRegisters
            numStacks
            maxStackLength
            programSpec(slug: $programSlug) {
              ...PuzzleDetails_programSpec
            }
          }
        }
      `}
      variables={{ hwSlug, programSlug, loggedIn }}
      render={({ props }) => {
        if (props?.hardwareSpec?.programSpec) {
          return <PuzzleDetails programSpec={props.hardwareSpec.programSpec} />;
        }

        return <NotFoundPage />;
      }}
    />
  );
};

export default PuzzleView;
