import React, { useState } from 'react';
import { TextField } from '@material-ui/core';
import Form from '@root/components/common/Form';
import LoadingButton from '@root/components/common/LoadingButton';

/**
 * A modal that allows for editing METADATA for a puzzle solution.
 * This allows changing the name and possibly other metadata in the future,
 * but not the source code!
 */
const EditPuzzleSolutionForm: React.FC<{
  fileName?: string;
  loading?: boolean;
  onSubmit: ({ fileName }: { fileName: string }) => void;
}> = ({ fileName, loading = false, onSubmit }) => {
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

export default EditPuzzleSolutionForm;
