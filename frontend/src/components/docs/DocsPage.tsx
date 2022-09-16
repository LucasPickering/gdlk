import React from "react";
import { Typography } from "@mui/material";
import { makeStyles } from "@mui/styles";
import IntroductionDocs from "./IntroductionDocs";
import HardwareDocs from "./hardware/HardwareDocs";
import LanguageDocs from "./language/LanguageDocs";
import { DocsContext } from "@root/state/docs";
import { hardwareUpgradeState } from "@root/state/user";
import { useRecoilValue } from "recoil";

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  docs: {
    padding: spacing(2),

    "& code": {
      color: palette.text.secondary,
    },
    "& pre": {
      backgroundColor: palette.background.paper,
      padding: spacing(1),
    },

    // Text styles
    "& h2": {
      marginTop: spacing(4),
    },
    "& h3": {
      marginTop: spacing(3),
    },
    "& h4": {
      marginTop: spacing(2),
    },
    "& h5, p": {
      marginTop: spacing(1),
    },

    "& ul": {
      margin: 0,
    },

    // Table styles
    "& table": {
      marginTop: spacing(2),
      border: `2px solid ${palette.divider}`,
      borderCollapse: "collapse",
    },
    "& th, td": {
      border: `1px solid ${palette.divider}`,
      padding: spacing(0.5),
    },
  },
}));

/**
 * All of the language docs content. This content is entirely static.
 */
const DocsPage: React.FC = () => {
  const localClasses = useLocalStyles();
  const hardwareSpec = useRecoilValue(hardwareUpgradeState);
  // We use this to selectively hide irrelevant docs, based on the hardware
  const context = {
    showStacks: hardwareSpec.numStacks > 0,
  };

  // This content is written from within the GDLK canon, i.e. from the
  // perspective that GDLK is a real language and the reader owns a GDLKx PC.
  return (
    <DocsContext.Provider value={context}>
      <div className={localClasses.docs}>
        <Typography variant="h1">GDLK Documentation</Typography>
        <IntroductionDocs />
        <HardwareDocs />
        <LanguageDocs />
      </div>
    </DocsContext.Provider>
  );
};

export default DocsPage;
