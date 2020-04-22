import React from 'react';
import { makeStyles, ListItem, ListItemText } from '@material-ui/core';
import clsx from 'clsx';
import NavLink from 'components/common/NavLink';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  listItem: {
    padding: 0,
  },
  link: {
    width: '100%',
    height: '100%',
    padding: `${spacing(1)}px ${spacing(2)}px`,
    color: palette.text.primary,
    '&:hover': {
      textDecoration: 'none',
    },
  },
  active: {
    color: palette.primary.main,
  },
}));

/**
 * A link in the navigation drawer
 */
const DrawerLink: React.FC<React.ComponentProps<typeof NavLink>> = ({
  className,
  children,
  ...rest
}) => {
  const localClasses = useLocalStyles();
  return (
    <ListItem className={localClasses.listItem} button>
      <NavLink
        className={clsx(localClasses.link, className)}
        activeClassName={localClasses.active}
        {...rest}
      >
        <ListItemText primary={children} />
      </NavLink>
    </ListItem>
  );
};

export default DrawerLink;
