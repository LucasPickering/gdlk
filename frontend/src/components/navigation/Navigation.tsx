import {
  AppBar,
  Toolbar,
  makeStyles,
  SwipeableDrawer,
  IconButton,
  List,
} from '@material-ui/core';
import { Menu as IconMenu } from '@material-ui/icons';
import React, { useState } from 'react';
import HeaderLink from './HeaderLink';
import DrawerLink from './DrawerLink';
import useScreenSize from 'hooks/useScreenSize';

const LINKS = [
  {
    to: '/',
    label: 'Home',
    exact: true,
  },
  {
    to: '/terminal',
    label: 'Terminal',
    exact: true,
  },
];

const useLocalStyles = makeStyles(({ spacing }) => ({
  drawer: {
    width: 150,
  },
  grow: {
    flexGrow: 1,
  },
  newMatchButton: {
    margin: spacing(1),
  },
}));

/**
 * Site-wide navigation controls. Will be a bar on large screens, and a drawer
 * on small ones.
 */
const Navigation: React.FC = () => {
  const localClasses = useLocalStyles();
  const drawerNavEnabled = useScreenSize() === 'small';

  const [drawerOpen, setDrawerOpen] = useState(false);
  const openDrawer = (): void => setDrawerOpen(true);
  const closeDrawer = (): void => setDrawerOpen(false);

  return (
    <AppBar position="static" color="default">
      <Toolbar component="nav" variant="dense">
        {drawerNavEnabled ? (
          <IconButton
            color="inherit"
            aria-label="open drawer"
            onClick={openDrawer}
            edge="start"
          >
            <IconMenu />
          </IconButton>
        ) : (
          LINKS.map(({ to, label, exact }) => (
            <HeaderLink key={to} to={to} exact={exact}>
              {label}
            </HeaderLink>
          ))
        )}
        <div className={localClasses.grow} />
      </Toolbar>

      {drawerNavEnabled && (
        <SwipeableDrawer
          open={drawerOpen}
          onOpen={openDrawer}
          onClose={closeDrawer}
        >
          <List className={localClasses.drawer} component="nav">
            {LINKS.map(({ to, label, exact }) => (
              <DrawerLink key={to} to={to} exact={exact} onClick={closeDrawer}>
                {label}
              </DrawerLink>
            ))}
          </List>
        </SwipeableDrawer>
      )}
    </AppBar>
  );
};

export default Navigation;
