import React from 'react';
import { LangValue } from 'state/ide';
import { makeStyles, Typography } from '@material-ui/core';
import { range } from 'lodash-es';
import clsx from 'clsx';
import LangValueDisplay from './LangValueDisplay';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  bufferDisplay: {
    display: 'flex',
    flexDirection: 'column',

    // Spacing between multiple buffers
    '&:not(:first-child)': {
      paddingLeft: spacing(1),
    },
  },

  buffer: {
    display: 'flex',
  },
  bufferCells: {
    display: 'flex',
    flexDirection: 'column',
    padding: spacing(0.5),
    border: `2px solid ${palette.divider}`,
    overflowY: 'auto',

    '& + &': {
      borderLeft: 0,
    },
  },
  bufferCellsInverted: {
    flexDirection: 'column-reverse',
  },
}));

const Buffer: React.FC<{
  values: readonly LangValue[];
  maxLength: number;
  invert: boolean;
}> = ({ values, maxLength, invert }) => {
  const localClasses = useLocalStyles();

  return (
    <div
      className={clsx(localClasses.bufferCells, {
        [localClasses.bufferCellsInverted]: invert,
      })}
    >
      {range(maxLength).map((i) => (
        <LangValueDisplay key={i} value={values[i]} />
      ))}
    </div>
  );
};

interface Props {
  className?: string;
  label: string;
  values: readonly LangValue[];
  secondaryValues?: readonly LangValue[];
  maxLength: number;
  invert: boolean;
}

const BufferDisplay = ({
  className,
  label,
  values,
  secondaryValues,
  maxLength,
  invert,
}: Props): React.ReactElement => {
  const localClasses = useLocalStyles();

  return (
    <div className={clsx(localClasses.bufferDisplay, className)}>
      <Typography variant="body2">{label}</Typography>

      <div className={localClasses.buffer}>
        <Buffer values={values} maxLength={maxLength} invert={invert} />
        {secondaryValues && (
          <Buffer
            values={secondaryValues}
            maxLength={maxLength}
            invert={invert}
          />
        )}
      </div>
    </div>
  );
};

BufferDisplay.defaultProps = {
  invert: false,
};

export default BufferDisplay;
