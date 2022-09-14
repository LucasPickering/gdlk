import React from "react";
import { LangValue } from "@root/state/ide";
import { Typography } from "@mui/material";
import { makeStyles } from "@mui/styles";
import clsx from "clsx";

const useLocalStyles = makeStyles(() => ({
  langValueDisplay: {
    width: 40, // TODO we'll need to tweak max value to make it fit into this
    textAlign: "right",
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
      {value ?? "-"}
    </Typography>
  );
};

export default LangValueDisplay;
