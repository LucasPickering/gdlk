import { makeStyles, Typography } from '@material-ui/core';
import React from 'react';
import ButtonLink from './common/ButtonLink';
import Link from './common/Link';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  root: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    height: '100%',
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
      borderLeftWidth: 1,
      borderLeftStyle: 'solid',
      borderLeftColor: palette.divider,
    },
  },
}));

/**
 * Container for all content on the page. This is used for everything but the
 * terminal.
 */
const PageContainer: React.FC = () => {
  const localClasses = useLocalStyles();

  // Only render the page if user data is loaded
  return (
    <div className={localClasses.root}>
      <ButtonLink to="/terminal" variant="contained" color="primary">
        Go to Terminal
      </ButtonLink>

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
