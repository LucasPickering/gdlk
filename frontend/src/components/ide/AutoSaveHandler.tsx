import React, { useState, useContext, useEffect } from 'react';
import { graphql } from 'react-relay';
import { Snackbar } from '@material-ui/core';
import { Alert } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import { useMutation } from 'relay-hooks';
import { AutoSaveHandler_Mutation } from './__generated__/AutoSaveHandler_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { AutoSaveHandler_puzzleSolution } from './__generated__/AutoSaveHandler_puzzleSolution.graphql';
import useDebouncedValue from 'hooks/useDebouncedValue';

const savePuzzleSolutionMutation = graphql`
  mutation AutoSaveHandler_Mutation($id: ID!, $sourceCode: String!) {
    updatePuzzleSolution(input: { id: $id, sourceCode: $sourceCode }) {
      puzzleSolutionEdge {
        node {
          id
          sourceCode
        }
      }
    }
  }
`;

/**
 * Component to automatically save edits to source code. Includes a snack bar
 * status indicator.
 */
const AutoSaveHandler: React.FC<{
  puzzleSolution: AutoSaveHandler_puzzleSolution;
  relay: RelayProp;
}> = ({ puzzleSolution }) => {
  const { sourceCode } = useContext(IdeContext);
  // Debounce changes so we're not sending constant requests
  const debouncedSourceCode = useDebouncedValue(sourceCode, 1000);
  const [mutate, { loading }] = useMutation<AutoSaveHandler_Mutation>(
    savePuzzleSolutionMutation
  );
  const [saveState, setSaveState] = useState<'success' | 'error' | undefined>();

  useEffect(() => {
    // If the user has changed the source code, save the new value
    if (debouncedSourceCode !== puzzleSolution.sourceCode) {
      mutate({
        variables: {
          id: puzzleSolution.id,
          sourceCode: debouncedSourceCode,
        },
        onCompleted: () => setSaveState('success'),
        onError: () => setSaveState('error'),
      });
    }
  }, [
    debouncedSourceCode,
    puzzleSolution.sourceCode,
    puzzleSolution.id,
    mutate,
  ]);

  return (
    <>
      <Snackbar
        open={!loading && saveState === 'success'}
        onClose={() => setSaveState(undefined)}
      >
        <Alert severity="success">Solution saved</Alert>
      </Snackbar>

      <Snackbar
        open={!loading && saveState === 'error'}
        onClose={() => setSaveState(undefined)}
      >
        <Alert severity="error">Error saving program</Alert>
      </Snackbar>
    </>
  );
};

export default createFragmentContainer(AutoSaveHandler, {
  puzzleSolution: graphql`
    fragment AutoSaveHandler_puzzleSolution on PuzzleSolutionNode {
      id
      sourceCode
    }
  `,
});
