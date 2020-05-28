import { makeStyles } from '@material-ui/core';
import React from 'react';
import { noop } from 'lodash-es';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ spacing }) => ({
  formPaper: {
    alignSelf: 'center',
  },
  small: { width: 200 },
  medium: { width: 300 },
  innerBox: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'left',

    '& .MuiFormControl-root': {
      width: '100%',
    },
    '& > * + *': {
      marginTop: spacing(1),
    },
  },
}));

interface Props {
  className?: string;
  size: 'small' | 'medium';
  onSubmit: () => void;
}

const Form: React.FC<{
  className?: string;
  size: 'small' | 'medium';
  onSubmit: () => void;
}> = ({ className, size, onSubmit, children }): React.ReactElement => {
  const localClasses = useLocalStyles();

  return (
    <form
      className={clsx(localClasses[size], className)}
      onSubmit={
        onSubmit &&
        ((event) => {
          onSubmit();
          event.preventDefault(); // Don't reload the page
        })
      }
    >
      <div className={localClasses.innerBox}>{children}</div>
    </form>
  );
};

Form.defaultProps = {
  size: 'medium',
  onSubmit: noop,
};

export default Form;
