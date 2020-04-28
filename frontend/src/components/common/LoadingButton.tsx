import { makeStyles, Button, CircularProgress } from '@material-ui/core';
import React from 'react';

const useLocalStyles = makeStyles({
  loading: {
    position: 'absolute',
    top: '50%',
    left: '50%',
    marginTop: -12,
    marginLeft: -12,
  },
});

interface Props {
  loading: boolean;
}

const LoadingButton: React.FC<Props & React.ComponentProps<typeof Button>> & {
  defaultProps: Partial<Props>;
} = ({ loading, color, disabled, children, ...rest }) => {
  const localClasses = useLocalStyles();
  return (
    <Button color={color} disabled={disabled || loading} {...rest}>
      {children}
      {loading && (
        <CircularProgress
          className={localClasses.loading}
          color={color === 'default' ? 'inherit' : color}
          size={24}
        />
      )}
    </Button>
  );
};

LoadingButton.defaultProps = {
  loading: false,
};

export default LoadingButton;
