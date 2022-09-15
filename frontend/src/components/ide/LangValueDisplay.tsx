import React from "react";
import { LangValue } from "@root/state/ide";
import { Typography } from "@mui/material";

interface Props {
  value?: LangValue;
}

/**
 * A simple component to display a `LangValue`. Just displays the value, or
 * a placeholder if not present.
 * @param value The value to display
 */
const LangValueDisplay: React.FC<Props> = ({ value }) => (
  <Typography
    width={40} // TODO we'll need to tweak max value to make it fit into this
    textAlign="right"
    lineHeight={1.1}
    component="span"
  >
    {value ?? "-"}
  </Typography>
);

export default LangValueDisplay;
