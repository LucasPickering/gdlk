import React, { useState } from 'react';
import { TextField } from '@material-ui/core';
import Form from 'components/common/Form';
import LoadingButton from 'components/common/LoadingButton';

/**
 * A modal that allows for editing METADATA for a user program (i.e. a solution).
 * This allows changing the name and possibly other metadata in the future,
 * but not the source code!
 *
 * @param fileName If provided, will be the starting file name value
 * @param loading If the save action is loading. Will show a loading state on
 *  the submit button.
 * @param onSubmit Callback for when the submit button is pressed
 */
const EditUserProgramForm: React.FC<{
  fileName?: string;
  loading: boolean;
  onSubmit: ({ fileName }: { fileName: string }) => void;
}> = ({ fileName, loading, onSubmit }) => {
  const [currentFileName, setCurrentFileName] = useState<string>(
    fileName ?? ''
  );
  const creatingNew = !fileName;

  return (
    <Form size="small" onSubmit={() => onSubmit({ fileName: currentFileName })}>
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
