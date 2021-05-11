import React from 'react';
import { FileCopy as IconFileCopy } from '@material-ui/icons';
import { graphql } from 'react-relay';
import { useMutation } from 'relay-hooks';
import { CopyPuzzleSolutionButton_Mutation } from './__generated__/CopyPuzzleSolutionButton_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { CopyPuzzleSolutionButton_userProgram } from './__generated__/CopyPuzzleSolutionButton_userProgram.graphql';
import IconButton from 'components/common/IconButton';

const copyPuzzleSolutionMutation = graphql`
  mutation CopyPuzzleSolutionButton_Mutation($id: ID!) {
    copyPuzzleSolution(input: { id: $id }) {
      puzzleSolutionEdge {
        node {
          name
        }
      }
    }
  }
`;

/**
 * A button that duplicates an existing user program.
 *
 * @param programSpecId The ID of the program that owns this solution
 * @param userProgram The user program being copied
 */
const CopyPuzzleSolutionButton: React.FC<{
  programSpecId: string;
  userProgram: CopyPuzzleSolutionButton_userProgram;
  relay: RelayProp;
}> = ({ programSpecId, userProgram }) => {
  const [mutate, { loading }] = useMutation<CopyPuzzleSolutionButton_Mutation>(
    copyPuzzleSolutionMutation
  );

  return (
    <IconButton
      aria-label="Copy solution"
      loading={loading}
      onClick={() => {
        mutate({
          variables: {
            id: userProgram.id,
          },
          configs: [
            // Update the list of programs in the parent after modification
            {
              type: 'RANGE_ADD',
              parentID: programSpecId,
              connectionInfo: [
                {
                  key: 'PuzzleSolutionsCard_userPrograms',
                  rangeBehavior: 'append',
                },
              ],
              edgeName: 'puzzleSolutionEdge',
            },
          ],
        });
      }}
    >
      <IconFileCopy />
    </IconButton>
  );
};

export default createFragmentContainer(CopyPuzzleSolutionButton, {
  userProgram: graphql`
    fragment CopyPuzzleSolutionButton_userProgram on PuzzleSolutionNode {
      id
    }
  `,
});
