import React from 'react';
import graphql from 'babel-plugin-relay/macro';
import ProgramSpecList from './ProgramSpecList';
import { HardwareSpecViewQuery } from './__generated__/HardwareSpecViewQuery.graphql';
import { useParams } from 'react-router-dom';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';

interface RouteParams {
  hwSlug: string;
}

const HardwareSpecView: React.FC = () => {
  const { hwSlug } = useParams<RouteParams>();

  return (
    <QueryResult<HardwareSpecViewQuery>
      query={graphql`
        query HardwareSpecViewQuery($hwSlug: String!) {
          hardwareSpec(slug: $hwSlug) {
            id
            ...ProgramSpecList_hardwareSpec @arguments(count: 1)
          }
        }
      `}
      variables={{ hwSlug }}
      render={({ props }) => {
        if (props?.hardwareSpec) {
          return <ProgramSpecList hardwareSpec={props.hardwareSpec} />;
        }

        return <NotFoundPage />;
      }}
    />
  );
};

export default HardwareSpecView;
