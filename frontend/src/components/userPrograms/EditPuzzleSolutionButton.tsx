import React, { useState } from 'react';
import {
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
} from '@material-ui/core';
import { Edit as IconEdit } from '@material-ui/icons';
import EditPuzzleSolutionForm from './EditPuzzleSolutionForm';
import { graphql } from 'react-relay';
import { useMutation } from 'relay-hooks';
import { EditPuzzleSolutionButton_Mutation } from './__generated__/EditPuzzleSolutionButton_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { EditPuzzleSolutionButton_puzzleSolution } from './__generated__/EditPuzzleSolutionButton_puzzleSolution.graphql';

const updatePuzzleSolutionMutation = graphql`
  mutation EditPuzzleSolutionButton_Mutation($id: ID!, $name: String!) {
    updatePuzzleSolution(input: { id: $id, name: $name }) {
      puzzleSolutionEdge {
        node {
          name
        }
      }
    }
  }
`;

/**
 * A button to open a modal that allows the user to edit the METADATA for a
 * puzzle solution (NOT the source code).
 *
 * @param puzzleSolution The puzzle solution being edited
 */
const EditPuzzleSolutionButton: React.FC<{
  puzzleSolution: EditPuzzleSolutionButton_puzzleSolution;
  relay: RelayProp;
}> = ({ puzzleSolution }) => {
  const [mutate, { loading }] = useMutation<EditPuzzleSolutionButton_Mutation>(
    updatePuzzleSolutionMutation
  );
  const [modalOpen, setModalOpen] = useState<boolean>(false);

  return (
    <>
      <IconButton aria-label="Edit solution" onClick={() => setModalOpen(true)}>
        <IconEdit />
      </IconButton>

      <Dialog
        aria-labelledby="edit-user-program-dialog-title"
        open={modalOpen}
        onClose={() => setModalOpen(false)}
      >
        <DialogTitle id="edit-user-program-dialog-title">
          Edit solution
        </DialogTitle>
        <DialogContent>
          <EditPuzzleSolutionForm
            name={puzzleSolution.name}
            loading={loading}
            onSubmit={({ name }) => {
              mutate({
                variables: {
                  id: puzzleSolution.id,
                  name,
                },
                onCompleted: () => setModalOpen(false),
              });
            }}
          />
        </DialogContent>
      </Dialog>
    </>
  );
};

export default createFragmentContainer(EditPuzzleSolutionButton, {
  puzzleSolution: graphql`
    fragment EditPuzzleSolutionButton_puzzleSolution on PuzzleSolutionNode {
      id
      name
    }
  `,
});
