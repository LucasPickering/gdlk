import React, { useContext } from 'react';
import { graphql } from 'react-relay';
import { PuzzleViewQuery } from './__generated__/PuzzleViewQuery.graphql';
import { useParams } from 'react-router-dom';
import PuzzleDetails from './PuzzleDetails';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import { UserContext } from 'state/user';

interface RouteParams {
  hardwareSpecSlug: string;
  puzzleSlug: string;
}

const PuzzleView: React.FC = () => {
  const { hardwareSpecSlug, puzzleSlug } = useParams<RouteParams>();
  const { loggedIn } = useContext(UserContext);

  return (
    <QueryResult<PuzzleViewQuery>
      query={graphql`
        query PuzzleViewQuery(
          $hardwareSpecSlug: String!
          $puzzleSlug: String!
        ) {
          hardwareSpec(slug: $hardwareSpecSlug) {
            slug
            numRegisters
            numStacks
            maxStackLength
          }
          puzzle(slug: $puzzleSlug) {
            ...PuzzleDetails_puzzle
          }
        }
      `}
      variables={{ hardwareSpecSlug, puzzleSlug }}
      render={({ props }) => {
        if (props?.hardwareSpec?.puzzle) {
          return <PuzzleDetails puzzle={props.puzzle} />;
        }

        return <NotFoundPage />;
      }}
    />
  );
};

export default PuzzleView;
