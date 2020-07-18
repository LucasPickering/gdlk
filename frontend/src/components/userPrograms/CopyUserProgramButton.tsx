import React from 'react';
import { FileCopy as IconFileCopy } from '@material-ui/icons';
import graphql from 'babel-plugin-relay/macro';
import { useMutation } from 'relay-hooks';
import { CopyUserProgramButton_Mutation } from './__generated__/CopyUserProgramButton_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { CopyUserProgramButton_userProgram } from './__generated__/CopyUserProgramButton_userProgram.graphql';
import IconButton from 'components/common/IconButton';

const updateUserProgramMutation = graphql`
  mutation CopyUserProgramButton_Mutation($id: ID!) {
    copyUserProgram(input: { id: $id }) {
      userProgramEdge {
        node {
          fileName
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
const CopyUserProgramButton: React.FC<{
  programSpecId: string;
  userProgram: CopyUserProgramButton_userProgram;
  relay: RelayProp;
}> = ({ programSpecId, userProgram }) => {
  const [mutate, { loading }] = useMutation<CopyUserProgramButton_Mutation>(
    updateUserProgramMutation
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
                  key: 'UserProgramsCard_userPrograms',
                  rangeBehavior: 'append',
                },
              ],
              edgeName: 'userProgramEdge',
            },
          ],
        });
      }}
    >
      <IconFileCopy />
    </IconButton>
  );
};

export default createFragmentContainer(CopyUserProgramButton, {
  userProgram: graphql`
    fragment CopyUserProgramButton_userProgram on UserProgramNode {
      id
    }
  `,
});
