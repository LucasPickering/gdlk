import React, { useState, useContext, useEffect } from 'react';
import graphql from 'babel-plugin-relay/macro';
import { Snackbar } from '@material-ui/core';
import { Alert } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import { useMutation } from 'relay-hooks';
import { AutoSaveHandler_Mutation } from './__generated__/AutoSaveHandler_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { AutoSaveHandler_userProgram } from './__generated__/AutoSaveHandler_userProgram.graphql';
import useDebouncedValue from 'hooks/useDebouncedValue';

const saveUserProgramMutation = graphql`
  mutation AutoSaveHandler_Mutation($id: ID!, $sourceCode: String!) {
    updateUserProgram(input: { id: $id, sourceCode: $sourceCode }) {
      userProgramEdge {
        node {
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
  userProgram: AutoSaveHandler_userProgram;
  relay: RelayProp;
}> = ({ userProgram }) => {
  const { sourceCode } = useContext(IdeContext);
  // Debounce changes so we're not sending constant requests
  const debouncedSourceCode = useDebouncedValue(sourceCode, 1000);
  const [mutate, { loading }] = useMutation<AutoSaveHandler_Mutation>(
    saveUserProgramMutation
  );
  const [saveState, setSaveState] = useState<'success' | 'error' | undefined>();

  useEffect(() => {
    // If the user has changed the source code, save the new value
    if (debouncedSourceCode !== userProgram.sourceCode) {
      mutate({
        variables: {
          id: userProgram.id,
          sourceCode: debouncedSourceCode,
        },
        onCompleted: () => setSaveState('success'),
        onError: () => setSaveState('error'),
      });
    }
  }, [debouncedSourceCode, userProgram.sourceCode, userProgram.id, mutate]);

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
  userProgram: graphql`
    fragment AutoSaveHandler_userProgram on UserProgramNode {
      id
      sourceCode
    }
  `,
});
