import {
  AppBar,
  Toolbar,
  makeStyles,
  SwipeableDrawer,
  IconButton,
  List,
  Button,
} from '@material-ui/core';
import {
  Menu as IconMenu,
  ArrowBack as IconArrowBack,
} from '@material-ui/icons';
import React, { useState } from 'react';
import HeaderLink from './HeaderLink';
import DrawerLink from './DrawerLink';
import useScreenSize from 'hooks/useScreenSize';
import { Link } from 'react-router-dom';

const LINKS = [
  {
    to: '/',
    label: 'Home',
    exact: true,
  },
  { to: '/docs', label: 'Docs' },
];

const useLocalStyles = makeStyles(({ spacing }) => ({
  drawer: {
    width: 150,
  },
  drawerButton: {
    marginRight: spacing(1),
  },
  grow: {
    flexGrow: 1,
  },
}));

/**
 * Site-wide navigation controls. Will be a bar on large screens, and a drawer
 * on small ones.
 *
 * @param backLink If given, a button will be shown to go to a previous page
 */
const Navigation: React.FC<{
  backLink?: { to: string; label: string };
}> = ({ backLink }) => {
  const localClasses = useLocalStyles();
  const drawerNavEnabled = useScreenSize() === 'small';

  const [drawerOpen, setDrawerOpen] = useState(false);
  const openDrawer = (): void => setDrawerOpen(true);
  const closeDrawer = (): void => setDrawerOpen(false);

  return (
    <AppBar position="static" color="default">
      <Toolbar component="nav" variant="dense">
        {/* Drawer nav, only shown on small screens */}
        {drawerNavEnabled && (
          <>
            <SwipeableDrawer
              open={drawerOpen}
              onOpen={openDrawer}
              onClose={closeDrawer}
            >
              <List className={localClasses.drawer} component="nav">
                {LINKS.map(({ to, label, exact }) => (
                  <DrawerLink
                    key={to}
                    to={to}
                    exact={exact}
                    onClick={closeDrawer}
                  >
                    {label}
                  </DrawerLink>
                ))}
              </List>
            </SwipeableDrawer>
            <IconButton
              className={localClasses.drawerButton}
              color="inherit"
              aria-label="open drawer"
              onClick={openDrawer}
              edge="start"
            >
              <IconMenu />
            </IconButton>
          </>
        )}

        {backLink && (
          <Button
            startIcon={<IconArrowBack />}
            color="primary"
            variant="outlined"
            component={Link}
            // This prop gets forwarded to Link
            to={backLink.to}
          >
            {backLink.label}
          </Button>
        )}

        {/* Normal list of links, shown on medium-large screens */}
        {!drawerNavEnabled &&
          LINKS.map(({ to, label, exact }) => (
            <HeaderLink key={to} to={to} exact={exact}>
              {label}
            </HeaderLink>
          ))}

        <div className={localClasses.grow} />
      </Toolbar>
    </AppBar>
  );
};

export default Navigation;
