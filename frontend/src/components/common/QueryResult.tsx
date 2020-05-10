import React from 'react';
import { CircularProgress } from '@material-ui/core';
import { useQuery } from 'relay-hooks';
import { OperationType, GraphQLTaggedNode } from 'relay-runtime';

interface Props<TOperation extends OperationType> {
  query: GraphQLTaggedNode | null | undefined;
  variables: TOperation['variables'];
  render: (renderProps: {
    props: TOperation['response'];
    retry: () => void;
  }) => React.ReactElement | null;
  renderError: (error: Error) => React.ReactElement | null;
}

/**
 * A helper component to render the results of queries. This handles the loading
 * and error states automatically.
 */
const QueryResult = <T extends OperationType>({
  query,
  variables,
  render,
  renderError,
}: Props<T>): React.ReactElement | null => {
  const { props, error, retry } = useQuery(query, variables);

  if (error) {
    return renderError(error);
  }

  if (props) {
    return render({ props, retry });
  }

  return <CircularProgress />;
};

QueryResult.defaultProps = {
  renderError: () => <div>An error occurred.</div>,
};

export default QueryResult;
