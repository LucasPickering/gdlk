import React from 'react';
import { makeStyles, Typography, Theme } from '@material-ui/core';

type Level = 2 | 3 | 4 | 5;

// We need to statically map these so that TS can handle the types
const HEADER_MAPPINGS: Record<Level, 'h2' | 'h3' | 'h4' | 'h5'> = {
  2: 'h2',
  3: 'h3',
  4: 'h4',
  5: 'h5',
};

// We need to override the key type to make the dynamic classes work
const useLocalStyles = makeStyles<Theme, string>(({ spacing }) => ({
  docsSection: {},
  'docsSectionContent--3': {
    padding: `0 ${spacing(2)}px`,
  },
}));

/**
 * One section of the docs, with a dynamic header level.
 */
const DocsSection: React.FC<{
  id?: string;
  level: Level;
  title: string;
}> = ({ id, level, title, children }) => {
  const localClasses = useLocalStyles();

  return (
    <section id={id} className={localClasses.docsSection}>
      <Typography variant={HEADER_MAPPINGS[level]}>{title}</Typography>
      <div className={localClasses[`docsSectionContent--${level}`]}>
        {children}
      </div>
    </section>
  );
};

export default DocsSection;
