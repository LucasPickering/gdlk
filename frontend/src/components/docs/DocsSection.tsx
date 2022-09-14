import React from "react";
import { Typography, Theme } from "@mui/material";
import { makeStyles } from "@mui/styles";
import { Link as IconLink } from "@mui/icons-material";
import Link from "@root/components/common/Link";

type Level = 2 | 3 | 4 | 5;

// We need to statically map these so that TS can handle the types
const HEADER_MAPPINGS: Record<Level, "h2" | "h3" | "h4" | "h5"> = {
  2: "h2",
  3: "h3",
  4: "h4",
  5: "h5",
};

// We need to override the key type to make the dynamic classes work
const useLocalStyles = makeStyles<Theme>(({ spacing }) => ({
  docsSection: {},
  "docsSectionContent--3": {
    padding: `0 ${spacing(2)}`,
  },
  docsSectionHeader: {
    display: "flex",
    alignItems: "center",
  },
  docsSectionLink: {
    fontSize: 0,
    marginLeft: spacing(1),
  },
}));

/**
 * One section of the docs, with a dynamic header level.
 */
const DocsSection: React.FC<{
  id?: string;
  level: Level;
  title: string;
  children?: React.ReactNode;
}> = ({ id, level, title, children }) => {
  const localClasses = useLocalStyles();

  return (
    <section id={id} className={localClasses.docsSection}>
      <Typography
        className={localClasses.docsSectionHeader}
        variant={HEADER_MAPPINGS[level]}
      >
        {title}
        {id && (
          <Link className={localClasses.docsSectionLink} to={`#${id}`}>
            <IconLink />
          </Link>
        )}
      </Typography>
      <div className={localClasses[`docsSectionContent--${level}`]}>
        {children}
      </div>
    </section>
  );
};

export default DocsSection;
