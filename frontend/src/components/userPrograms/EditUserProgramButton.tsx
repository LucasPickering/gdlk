import React, { useState } from 'react';
import {
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
} from '@material-ui/core';
import { Edit as IconEdit } from '@material-ui/icons';
import EditUserProgramForm from './EditUserProgramForm';
import graphql from 'babel-plugin-relay/macro';
import { useMutation } from 'relay-hooks';
import { EditUserProgramButton_Mutation } from './__generated__/EditUserProgramButton_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { EditUserProgramButton_userProgram } from './__generated__/EditUserProgramButton_userProgram.graphql';

const updateUserProgramMutation = graphql`
  mutation EditUserProgramButton_Mutation($id: ID!, $fileName: String!) {
    updateUserProgram(input: { id: $id, fileName: $fileName }) {
      userProgramEdge {
        node {
          fileName
        }
      }
    }
  }
`;

/**
 * A button to open a modal that allows the user to edit the METADATA for a user
 * program (NOT the source code).
 *
 * @param userProgram The user program being edited
 */
const EditUserProgramButton: React.FC<{
  userProgram: EditUserProgramButton_userProgram;
  relay: RelayProp;
}> = ({ userProgram }) => {
  const [mutate, { loading }] = useMutation<EditUserProgramButton_Mutation>(
    updateUserProgramMutation
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
          <EditUserProgramForm
            fileName={userProgram.fileName}
            loading={loading}
            onSubmit={({ fileName }) => {
              mutate({
                variables: {
                  id: userProgram.id,
                  fileName,
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

export default createFragmentContainer(EditUserProgramButton, {
  userProgram: graphql`
    fragment EditUserProgramButton_userProgram on UserProgramNode {
      id
      fileName
    }
  `,
});
