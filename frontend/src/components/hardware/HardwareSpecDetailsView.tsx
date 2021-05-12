import React, { useContext } from 'react';
import { graphql } from 'react-relay';
import HardwareSpecDetails from './HardwareSpecDetails';
import { HardwareSpecDetailsViewQuery } from './__generated__/HardwareSpecDetailsViewQuery.graphql';
import { useParams } from 'react-router-dom';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import { UserContext } from 'state/user';

interface RouteParams {
  hardwareSpecSlug: string;
}

const HardwareSpecView: React.FC = () => {
  const { hardwareSpecSlug } = useParams<RouteParams>();
  const { loggedIn } = useContext(UserContext);

  return (
    <QueryResult<HardwareSpecDetailsViewQuery>
      query={graphql`
        query HardwareSpecDetailsViewQuery($hardwareSpecSlug: String!) {
          hardwareSpec(slug: $hardwareSpecSlug) {
            id
            ...HardwareSpecDetails_hardwareSpec
          }
        }
      `}
      variables={{ hardwareSpecSlug }}
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
