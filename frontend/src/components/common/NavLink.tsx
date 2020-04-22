import React from 'react';
import { NavLink as RouterNavLink } from 'react-router-dom';
import { makeStyles } from '@material-ui/core';
import clsx from 'clsx';
import { useLinkStyles } from './Link';

const useLocalStyles = makeStyles({
  active: {
    textDecoration: 'underline',
  },
});

const NavLink: React.FC<React.ComponentProps<typeof RouterNavLink>> = ({
  className,
  ...rest
}) => {
  const linkClasses = useLinkStyles();
  const localClasses = useLocalStyles();
  return (
    <RouterNavLink
      className={clsx(linkClasses.link, className)}
      activeClassName={localClasses.active}
      {...rest}
    />
  );
};

export default NavLink;
