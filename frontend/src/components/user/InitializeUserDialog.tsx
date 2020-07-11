import React, { useState, useContext } from 'react';
import {
  TextField,
  Dialog,
  DialogTitle,
  DialogContent,
} from '@material-ui/core';
import graphql from 'babel-plugin-relay/macro';
import { useMutation } from 'relay-hooks';
import { InitializeUserDialog_Mutation } from './__generated__/InitializeUserDialog_Mutation.graphql';
import Form from 'components/common/Form';
import LoadingButton from 'components/common/LoadingButton';
import { UserContext } from 'state/user';

const initializeUserMutation = graphql`
  mutation InitializeUserDialog_Mutation($username: String!) {
    initializeUser(input: { username: $username }) {
      userEdge {
        node {
          id
          username
        }
      }
    }
  }
`;

/**
 * The setup page that a user is sent to after first log in. This allows them
 * to set their username. This should only be shown for users that are logged
 * in but have not yet finished user setup.
 */
const InitializeUserDialog: React.FC = () => {
  const [currentUsername, setCurrentUsername] = useState<string>('');
  const [mutate, { loading }] = useMutation<InitializeUserDialog_Mutation>(
    initializeUserMutation
  );
  const { refetch } = useContext(UserContext);

  return (
    <Dialog aria-labelledby="user-setup-dialog-title" open>
      <DialogTitle id="user-setup-dialog-title">Choose a Username</DialogTitle>
      <DialogContent>
        <Form
          size="medium"
          onSubmit={() => {
            // TODO handle mutation errors
            mutate({
              variables: { username: currentUsername },
              onCompleted: refetch,
            });
          }}
        >
          <TextField
            autoFocus
            required
            label="Username"
            value={currentUsername}
            onChange={(e) => setCurrentUsername(e.target.value)}
          />
          <LoadingButton
            type="submit"
            variant="contained"
            color="primary"
            loading={loading}
          >
            Finish
          </LoadingButton>
        </Form>
      </DialogContent>
    </Dialog>
  );
};

export default InitializeUserDialog;
