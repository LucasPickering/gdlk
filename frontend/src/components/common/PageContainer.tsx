import { makeStyles, Typography } from '@material-ui/core';
import React from 'react';
import Link from './Link';
import Navigation from 'components/navigation/Navigation';
import clsx from 'clsx';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  pageContainer: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    height: '100%',
  },
  pageBody: {
    width: '100%',
  },
  pageBodyNotFullScreen: {
    maxWidth: 1280,
    padding: spacing(2),
    paddingBottom: 0,
  },
  pageBodyFullScreen: {
    height: '100%',
    overflowY: 'hidden',
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

interface Props {
  fullScreen: boolean;
  navProps: React.ComponentProps<typeof Navigation>;
}

/**
 * Container for all content on the page. This is used in the root to wrap all
 * pages.
 */
const PageContainer: React.FC<Props> & { defaultProps: Partial<Props> } = ({
  fullScreen,
  navProps,
  children,
}) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.pageContainer}>
      <Navigation {...navProps} />

      <div
        className={clsx(
          localClasses.pageBody,
          fullScreen
            ? localClasses.pageBodyFullScreen
            : localClasses.pageBodyNotFullScreen
        )}
      >
        {children}
      </div>

      {!fullScreen && (
        <footer className={localClasses.pageFooter}>
          <Typography variant="body2">
            Created by <Link to="https://github.com/JRMurr">John Murray</Link>{' '}
            and <Link to="https://lucaspickering.me">Lucas Pickering</Link>
          </Typography>
          <Link to="https://github.com/LucasPickering/gdlk">GitHub</Link>
        </footer>
      )}
    </div>
  );
};

PageContainer.defaultProps = {
  fullScreen: false,
  navProps: {},
};

export default PageContainer;
