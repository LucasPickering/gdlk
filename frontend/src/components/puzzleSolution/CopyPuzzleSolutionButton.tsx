import React from 'react';
import { FileCopy as IconFileCopy } from '@material-ui/icons';
import { graphql } from 'react-relay';
import { useMutation } from 'relay-hooks';
import { CopyPuzzleSolutionButton_Mutation } from './__generated__/CopyPuzzleSolutionButton_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { CopyPuzzleSolutionButton_puzzleSolution } from './__generated__/CopyPuzzleSolutionButton_puzzleSolution.graphql';
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
 * A button that duplicates an existing puzzle solution.
 *
 * @param puzzleId The ID of the puzzle that owns this solution
 * @param puzzleSolution The puzzle solution being copied
 */
const CopyPuzzleSolutionButton: React.FC<{
  puzzleId: string;
  puzzleSolution: CopyPuzzleSolutionButton_puzzleSolution;
  relay: RelayProp;
}> = ({ puzzleId, puzzleSolution }) => {
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
            id: puzzleSolution.id,
          },
          configs: [
            // Update the list of programs in the parent after modification
            {
              type: 'RANGE_ADD',
              parentID: puzzleId,
              connectionInfo: [
                {
                  key: 'PuzzleSolutionsCard_puzzleSolutions',
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
  puzzleSolution: graphql`
    fragment CopyPuzzleSolutionButton_puzzleSolution on PuzzleSolutionNode {
      id
    }
  `,
});
