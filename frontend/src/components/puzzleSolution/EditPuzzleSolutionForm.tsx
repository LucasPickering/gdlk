import React, { useState } from 'react';
import { TextField } from '@material-ui/core';
import Form from 'components/common/Form';
import LoadingButton from 'components/common/LoadingButton';

/**
 * A modal that allows for editing METADATA for a puzzle solution.
 * This allows changing the name and possibly other metadata in the future,
 * but not the source code!
 *
 * @param name If provided, will be the starting file name value
 * @param loading If the save action is loading. Will show a loading state on
 *  the submit button.
 * @param onSubmit Callback for when the submit button is pressed
 */
const EditPuzzleSolutionForm: React.FC<{
  name?: string;
  loading: boolean;
  onSubmit: ({ name }: { name: string }) => void;
}> = ({ name, loading, onSubmit }) => {
  const [currentName, setCurrentName] = useState<string>(name ?? '');
  const creatingNew = !name;

  return (
    <Form size="small" onSubmit={() => onSubmit({ name: currentName })}>
      <TextField
        autoFocus
        required
        label="File name"
        value={currentName}
        onChange={(e) => setCurrentName(e.target.value)}
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

export default EditPuzzleSolutionForm;
