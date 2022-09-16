import React from "react";
import { LangValue } from "@root/state/ide";
import { Box, StackProps, Typography } from "@mui/material";
import LangValueDisplay from "./LangValueDisplay";
import { Stack } from "@mui/system";
import { range } from "@root/util/general";

interface Props {
  className?: string;
  label: string;
  values: readonly LangValue[];
  maxLength: number;
  direction: StackProps["direction"];
  sx?: StackProps["sx"];
}

/**
 * Display a list of lang values. The display can be a grid layout (with
 * direction="row"), or a stack layout (direction="column-reverse"). There may
 * be further uses in the future too.
 *
 * If maxLength is given, empty slots in the buffer will be filled with a
 * placeholder symbol.
 */
const BufferDisplay = ({
  className,
  label,
  values,
  maxLength,
  direction,
  sx,
}: Props): React.ReactElement => (
  <Box className={className} sx={sx}>
    <Typography variant="body2">{label}</Typography>

    <Stack direction={direction} flexWrap="wrap">
      {range(0, maxLength).map((i) => (
        <LangValueDisplay key={i} value={values[i]} />
      ))}
    </Stack>
  </Box>
);

export default BufferDisplay;
