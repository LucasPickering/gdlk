import React from 'react';
import {
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  PropTypes,
} from '@material-ui/core';
import LoadingButton from './LoadingButton';

interface Props {
  open: boolean;
  // hidden label, used just for a11y
  label: string;
  title: string;
  loading: boolean;
  confirmText: string;
  confirmColor: PropTypes.Color;
  onConfirm?: () => void;
  onClose?: () => void;
}

const ConfirmationDialog: React.FC<Props> & {
  defaultProps: Partial<Props>;
} = ({
  open,
  label,
  title,
  loading,
  confirmText,
  confirmColor,
  onConfirm,
  onClose,
  children,
}) => {
  return (
    <Dialog aria-labelledby={`${label}-dialog`} open={open} onClose={onClose}>
      <DialogTitle id={`${label}-dialog`}>{title}</DialogTitle>
      <DialogContent>
        <DialogContentText>{children}</DialogContentText>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <LoadingButton
          loading={loading}
          color={confirmColor}
          variant="contained"
          onClick={onConfirm}
        >
          {confirmText}
        </LoadingButton>
      </DialogActions>
    </Dialog>
  );
};

ConfirmationDialog.defaultProps = {
  loading: false,
  confirmColor: 'primary',
};

export default ConfirmationDialog;
