import React from "react";
import { LangValue } from "@root/state/ide";
import { Box, Typography } from "@mui/material";
import { makeStyles } from "@mui/styles";
import { range } from "lodash-es";
import clsx from "clsx";
import LangValueDisplay from "./LangValueDisplay";

const useLocalStyles = makeStyles(({ palette }) => ({
  bufferDisplay: {
    flex: 1, // Size all buffer blocks evenly
    display: "flex",
    flexDirection: "column",

    // Border between multiple buffers
    "&:not(:first-child)": {
      borderTop: `1px solid ${palette.divider}`,
    },
  },

  buffer: {
    display: "flex",
  },
}));

const Buffer: React.FC<{
  values: readonly LangValue[];
  maxLength: number;
  invert: boolean;
}> = ({ values, maxLength, invert }) => (
  <Box
    display="flex"
    flexDirection={invert ? "row-reverse" : "row"}
    flexWrap="wrap"
  >
    {range(maxLength).map((i) => (
      <LangValueDisplay key={i} value={values[i]} />
    ))}
  </Box>
);

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
