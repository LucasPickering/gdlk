import React from 'react';
import {
  IconButton as MuiIconButton,
  Tooltip,
  CircularProgress,
  makeStyles,
} from '@material-ui/core';

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
  title?: string;
  loading: boolean;
}

/**
 * An IconButton with a few extensions:
 * - Loading prop
 * - Title prop, which automatically applies a tooltip (and aria-label)
 */
const IconButton = ({
  title,
  loading,
  color,
  children,
  ...rest
}: Props & React.ComponentProps<typeof MuiIconButton>): React.ReactElement => {
  const localClasses = useLocalStyles();

  const button = (
    <MuiIconButton
      aria-label={title}
      color={color}
      {...(loading ? { 'aria-busy': 'true', 'aria-live': 'polite' } : {})}
      {...rest}
    >
      {loading ? (
        <CircularProgress
          className={localClasses.loading}
          color={color === 'default' ? 'inherit' : color}
          size={24}
        />
      ) : (
        children
      )}
    </MuiIconButton>
  );

  // The span is needed so that the tooltip works while the button is disabled
  return title ? (
    <Tooltip title={title}>
      <span>{button}</span>
    </Tooltip>
  ) : (
    button
  );
};

IconButton.defaultProps = {
  loading: false,
} as Partial<Props>;

export default IconButton;
