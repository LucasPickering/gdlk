import React from 'react';
import { LangValue } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import clsx from 'clsx';

const useLocalStyles = makeStyles(() => ({
  langValueDisplay: {
    minWidth: 60, // Based on the min/max value
    textAlign: 'right',
    lineHeight: 1.1,
  },
}));

/**
 * A simple component to display a `LangValue`. Just displays the value, or
 * a placeholder if not present.
 * @param value The value to display
 */
const LangValueDisplay: React.FC<{ className?: string; value?: LangValue }> = ({
  className,
  value,
}) => {
  const localClasses = useLocalStyles();
  return (
    <Typography className={clsx(className, localClasses.langValueDisplay)}>
      {value ?? '-'}
    </Typography>
  );
};

export default LangValueDisplay;
