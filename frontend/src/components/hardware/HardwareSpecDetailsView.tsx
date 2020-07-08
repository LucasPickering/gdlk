import React, { useContext } from 'react';
import graphql from 'babel-plugin-relay/macro';
import HardwareSpecDetails from './HardwareSpecDetails';
import { HardwareSpecDetailsViewQuery } from './__generated__/HardwareSpecDetailsViewQuery.graphql';
import { useParams } from 'react-router-dom';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import { UserContext } from 'state/user';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();
  const { loggedIn } = useContext(UserContext);

  return (
    <QueryResult<HardwareSpecDetailsViewQuery>
      query={graphql`
        query HardwareSpecDetailsViewQuery(
          $loggedIn: Boolean!
          $hwSlug: String!
        ) {
          hardwareSpec(slug: $hwSlug) {
            id
            ...HardwareSpecDetails_hardwareSpec
          }
        }
      `}
      variables={{ loggedIn, hwSlug }}
      render={({ props }) => {
        if (props?.hardwareSpec) {
          return <HardwareSpecDetails hardwareSpec={props.hardwareSpec} />;
        }

        return <NotFoundPage />;
      }}
    />
  );
};

export default HardwareSpecView;
