import React, { useState } from 'react';
import { TextField } from '@material-ui/core';
import { useMutation } from 'relay-hooks';
import graphql from 'babel-plugin-relay/macro';
import Form from 'components/common/Form';
import LoadingButton from 'components/common/LoadingButton';
import { EditUserProgramForm_Mutation } from './__generated__/EditUserProgramForm_Mutation.graphql';

const saveUserProgramMutation = graphql`
  mutation EditUserProgramForm_Mutation(
    $programSpecId: ID!
    $fileName: String!
  ) {
    saveUserProgram(
      input: {
        programSpecId: $programSpecId
        fileName: $fileName
        sourceCode: "" # TODO
      }
    ) {
      userProgramEdge {
        node {
          fileName
        }
      }
    }
  }
`;

/**
 * A modal that allows for editing METADATA for a user program (i.e. a solution).
 * This allows changing the name and possibly other metadata in the future,
 * but not the source code!
 *
 * @param programSpecId The ID of the program that this solution belongs to
 * @param fileName If provided, will be the starting file name value
 * @param onCompleted Callback for a successful edit. Will be passed the new file name
 */
const EditUserProgramForm: React.FC<{
  programSpecId: string;
  fileName?: string;
  onCompleted: (newFileName: string) => void;
}> = ({ programSpecId, fileName, onCompleted }) => {
  const [mutate, { loading }] = useMutation<EditUserProgramForm_Mutation>(
    saveUserProgramMutation
  );
  const [currentFileName, setCurrentFileName] = useState<string>(
    fileName ?? ''
  );
  const creatingNew = !fileName;

  return (
    <Form
      size="small"
      onSubmit={() => {
        mutate({
          variables: { programSpecId, fileName: currentFileName },
          configs: [
            // Update the list of programs in the parent after modification
            {
              type: 'RANGE_ADD',
              parentID: programSpecId,
              connectionInfo: [
                {
                  key: 'UserProgramsTable_userPrograms',
                  rangeBehavior: 'append',
                },
              ],
              edgeName: 'userProgramEdge',
            },
          ],
          onCompleted: () => onCompleted(currentFileName),
        });
      }}
    >
      <TextField
        autoFocus
        required
        label="File name"
        value={currentFileName}
        onChange={(e) => setCurrentFileName(e.target.value)}
      />
      <LoadingButton
        type="submit"
        variant="contained"
        color="primary"
        loading={loading}
      >
        {creatingNew ? 'Create' : 'Save'}
      </LoadingButton>
    </Form>
  );
};

export default EditUserProgramForm;
