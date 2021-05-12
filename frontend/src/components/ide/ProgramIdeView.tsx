import React, { useState, useEffect } from 'react';
import { graphql } from 'react-relay';
import { ProgramIdeViewQuery } from './__generated__/ProgramIdeViewQuery.graphql';
import { useParams } from 'react-router-dom';
import ProgramIde from './ProgramIde';
import QueryResult from 'components/common/QueryResult';
import NotFoundPage from 'components/NotFoundPage';
import PageContainer from 'components/common/PageContainer';
import { CompilerWrapper } from 'util/compile';
import { CircularProgress } from '@material-ui/core';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
  fileName: string;
}

const query = graphql`
  query ProgramIdeViewQuery(
    $hardwareSpecSlug: String!
    $programSlug: String!
    $fileName: String!
  ) {
    ...ProgramIde_query
      @arguments(
        hardwareSpecSlug: $hardwareSpecSlug
        puzzleSlug: $puzzleSlug
        fileName: $fileName
      )
  }
`;

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const { hwSlug, programSlug, fileName } = useParams<RouteParams>();

  // Initialize the Compiler. Wasm imports have to be async until we get
  // webpack 5, so we want to block the entire page until it's imported.
  const [compilerInitialized, setCompilerInitialized] =
    useState<boolean>(false);
  useEffect(() => {
    CompilerWrapper.init().then(() => {
      setCompilerInitialized(true);
    });
  }, []);

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
          if (!compilerInitialized) {
            return <CircularProgress />;
          }

          if (props.hardwareSpec) {
            return <ProgramIde queryKey={props.hardwareSpec} />;
          }

          // TODO fix padding here
          return <NotFoundPage />;
        }}
      />
    </PageContainer>
  );
};

export default ProgramIdeView;
