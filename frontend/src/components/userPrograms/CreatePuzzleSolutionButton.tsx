import React, { useState } from 'react';
import { Dialog, DialogTitle, DialogContent, Button } from '@material-ui/core';
import { Add as IconAdd } from '@material-ui/icons';
import EditPuzzleSolutionForm from './EditPuzzleSolutionForm';
import { graphql } from 'react-relay';
import { useMutation } from 'relay-hooks';
import { CreatePuzzleSolutionButton_Mutation } from './__generated__/CreatePuzzleSolutionButton_Mutation.graphql';

const createPuzzleSolutionMutation = graphql`
  mutation CreatePuzzleSolutionButton_Mutation($puzzleId: ID!, $name: String!) {
    createPuzzleSolution(input: { puzzleId: $puzzleId, name: $name }) {
      puzzleSolutionEdge {
        node {
          name
        }
      }
    }
  }
`;

/**
 * A button to open a modal that creates a new a user program.
 *
 * @param className Optional CSS class name
 * @param puzzleId The ID of the program that will own the new user program
 */
const CreatePuzzleSolutionButton: React.FC<{
  className?: string;
  puzzleId: string;
}> = ({ className, puzzleId }) => {
  const [mutate, { loading }] =
    useMutation<CreatePuzzleSolutionButton_Mutation>(
      createPuzzleSolutionMutation
    );
  const [modalOpen, setModalOpen] = useState<boolean>(false);

  return (
    <>
      <Button
        className={className}
        startIcon={<IconAdd />}
        size="small"
        onClick={() => setModalOpen(true)}
      >
        New Solution
      </Button>
      <Dialog
        aria-labelledby="new-user-program-dialog-title"
        open={modalOpen}
        onClose={() => setModalOpen(false)}
      >
        <DialogTitle id="new-user-program-dialog-title">
          Create new solution
        </DialogTitle>
        <DialogContent>
          <EditPuzzleSolutionForm
            loading={loading}
            onSubmit={({ name }) => {
              mutate({
                variables: { puzzleId, name },
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
                onCompleted: () => setModalOpen(false),
              });
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default CreatePuzzleSolutionButton;
