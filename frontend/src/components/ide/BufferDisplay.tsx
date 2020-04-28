import React from 'react';
import { LangValue } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import { range } from 'lodash';
import LangValueDisplay from './LangValueDisplay';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  bufferDisplay: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    '&:not(:first-child)': {
      paddingLeft: spacing(1),
    },
  },
  bufferCells: {
    border: `1px solid ${palette.divider}`,
  },
  bufferCell: {
    '&:not(:first-child)': {
      borderTop: 0,
    },
  },
}));

const BufferDisplay: React.FC<{
  label: string;
  values: LangValue[];
  maxLength: number;
}> = ({ label, values, maxLength }) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.bufferDisplay}>
      <Typography variant="body2">{label}</Typography>

      <div className={localClasses.bufferCells}>
        {range(maxLength).map((i) => (
          <LangValueDisplay
            key={i}
            className={localClasses.bufferCell}
            value={values[i]}
          />
        ))}
      </div>
    </div>
  );
};

export default BufferDisplay;
