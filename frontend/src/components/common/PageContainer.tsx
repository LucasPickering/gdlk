import { makeStyles, Typography } from '@material-ui/core';
import React from 'react';
import Link from './Link';
import Navigation from 'components/navigation/Navigation';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  root: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    height: '100%',
  },
  pageBody: {
    margin: spacing(2),
  },
  pageFooter: {
    marginTop: 'auto',
    padding: spacing(2),
    display: 'flex',
    justifyContent: 'center',
    '& > *': {
      padding: `0px ${spacing(0.5)}px`,
    },
    '& > * + *': {
      borderLeftWidth: 2,
      borderLeftStyle: 'solid',
      borderLeftColor: palette.divider,
    },
  },
}));

/**
 * Container for all content on the page. This is used for everything but the
 * terminal.
 */
const PageContainer: React.FC = ({ children }) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.root}>
      <Navigation />

      <div className={localClasses.pageBody}>{children}</div>

      <footer className={localClasses.pageFooter}>
        <Typography variant="body2">
          Created by <Link to="https://github.com/JRMurr">John Murray</Link> and{' '}
          <Link to="https://lucaspickering.me">Lucas Pickering</Link>
        </Typography>
        <Link to="https://github.com/LucasPickering/gdlk">GitHub</Link>
      </footer>
    </div>
  );
};

export default PageContainer;
