import React from "react";
import { IconButton as MuiIconButton, Tooltip } from "@mui/material";

interface Props extends React.ComponentProps<typeof MuiIconButton> {
  title?: string;
  children?: React.ReactNode;
}

/**
 * An IconButton with a few extensions:
 * - Loading prop
 * - Title prop, which automatically applies a tooltip (and aria-label)
 */
const IconButton = ({
  title,
  color,
  children,
  ...rest
}: Props): React.ReactElement => {
  const button = (
    <MuiIconButton aria-label={title} color={color} {...rest}>
      {children}
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

export default IconButton;
