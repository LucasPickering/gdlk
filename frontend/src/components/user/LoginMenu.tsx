import React, { useContext, useState, useRef } from 'react';
import { UserContext } from 'state/user';
import { Button, Menu, MenuItem, IconButton } from '@material-ui/core';
import { AccountCircle as IconAccountCircle } from '@material-ui/icons';
import UnstyledLink from 'components/common/UnstyledLink';
import { useLocation } from 'react-router-dom';
import queryString from 'query-string';

/**
 * Component to hold either the login button or the account menu, depending on
 * whether the user is logged in or not.
 */
const LoginMenu: React.FC = () => {
  // We want unsafe here because we should show the logout button even if the
  // user hasn't finished setup
  const { loggedInUnsafe } = useContext(UserContext);
  const { pathname } = useLocation();
  const [open, setOpen] = useState<boolean>(false);
  const anchorEl = useRef<HTMLDivElement>(null);

  const logInParams = { next: pathname };

  return (
    <div ref={anchorEl}>
      {loggedInUnsafe ? (
        <>
          <IconButton
            aria-label="Account"
            aria-controls="account-menu"
            aria-haspopup="true"
            onClick={() => setOpen(true)}
          >
            <IconAccountCircle />
          </IconButton>
          <Menu
            id="account-menu"
            anchorEl={anchorEl.current}
            getContentAnchorEl={null}
            anchorOrigin={{
              vertical: 'bottom',
              horizontal: 'right',
            }}
            transformOrigin={{
              vertical: 'top',
              horizontal: 'right',
            }}
            open={open}
            onClose={() => setOpen(false)}
          >
            <MenuItem
              onClick={async () => {
                await fetch('/api/logout', { method: 'POST' });
                // fuckin yeet em back to the home page yeehaw
                window.location.assign('/');
              }}
            >
              Log Out
            </MenuItem>
          </Menu>
        </>
      ) : (
        <>
          <Button
            aria-controls="login-menu"
            aria-haspopup="true"
            color="primary"
            variant="contained"
            onClick={() => setOpen(true)}
          >
            Log In
          </Button>
          {/* TODO: get providers from api */}
          <Menu
            id="account-menu"
            anchorEl={anchorEl.current}
            getContentAnchorEl={null}
            anchorOrigin={{
              vertical: 'bottom',
              horizontal: 'right',
            }}
            transformOrigin={{
              vertical: 'top',
              horizontal: 'right',
            }}
            open={open}
            onClose={() => setOpen(false)}
          >
            <MenuItem
              component={UnstyledLink}
              to={`/api/oidc/google/redirect?${queryString.stringify(
                logInParams
              )}`}
            >
              Log In With Google
            </MenuItem>
          </Menu>
        </>
      )}
    </div>
  );
};

export default LoginMenu;
