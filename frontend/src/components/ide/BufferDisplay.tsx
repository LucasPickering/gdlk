import React from 'react';
import { LangValue } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import { range } from 'lodash';
import LangValueDisplay from './LangValueDisplay';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  bufferDisplay: {
    // Spacing between multiple buffers
    '&:not(:first-child)': {
      paddingLeft: spacing(1),
    },
  },

  buffers: {
    display: 'flex',
  },
  bufferCells: {
    padding: spacing(0.5),
    border: `2px solid ${palette.divider}`,

    '& + &': {
      borderLeft: 0,
    },
  },
}));

const Buffer: React.FC<{ values: readonly LangValue[]; maxLength: number }> = ({
  values,
  maxLength,
}) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.bufferCells}>
      {range(maxLength).map((i) => (
        <LangValueDisplay key={i} value={values[i]} />
      ))}
    </div>
  );
};

const BufferDisplay: React.FC<{
  label: string;
  values: readonly LangValue[];
  secondaryValues?: readonly LangValue[];
  maxLength: number;
}> = ({ label, values, secondaryValues, maxLength }) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.bufferDisplay}>
      <Typography variant="body2">{label}</Typography>

      <div className={localClasses.buffers}>
        <Buffer values={values} maxLength={maxLength} />
        {secondaryValues && (
          <Buffer values={secondaryValues} maxLength={maxLength} />
        )}
      </div>
    </div>
  );
};

export default BufferDisplay;
