import React from 'react';
import graphql from 'babel-plugin-relay/macro';
import { ProgramIdeViewQuery } from './__generated__/ProgramIdeViewQuery.graphql';
import { useParams } from 'react-router-dom';
import ProgramIde from './ProgramIde';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import PageContainer from 'components/common/PageContainer';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
  fileName: string;
}

const query = graphql`
  query ProgramIdeViewQuery(
    $hwSlug: String!
    $programSlug: String!
    $fileName: String!
  ) {
    hardwareSpec(slug: $hwSlug) {
      ...ProgramIde_hardwareSpec
        @arguments(programSlug: $programSlug, fileName: $fileName)
    }
  }
`;

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const { hwSlug, programSlug, fileName } = useParams<RouteParams>();

  return (
    <PageContainer
      fullScreen
      navProps={{
        backLink: {
          to: `/hardware/${hwSlug}/puzzles/${programSlug}`,
          label: 'Back to Puzzle',
        },
      }}
    >
      <QueryResult<ProgramIdeViewQuery>
        query={query}
        variables={{ hwSlug, programSlug, fileName }}
        render={({ props }) => {
          if (props.hardwareSpec) {
            return <ProgramIde hardwareSpec={props.hardwareSpec} />;
          }

          // TODO fix padding here
          return <NotFoundPage />;
        }}
      />
    </PageContainer>
  );
};

export default ProgramIdeView;
