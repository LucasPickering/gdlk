import React, { useState } from 'react';
import {
  Collapse,
  List,
  ListItem,
  ListItemText,
  makeStyles,
} from '@material-ui/core';
import UnstyledLink from './UnstyledLink';

const useLocalStyles = makeStyles(({ spacing }) => ({
  nested: {
    paddingLeft: spacing(4),
  },
}));

interface NavItem {
  label: string;
  to?: React.ComponentProps<typeof UnstyledLink>['to'];
  onClick?: () => void;
  children?: React.ReactChild | React.ReactChildren;
}

const NavMenu: React.FC<
  {
    items: NavItem[];
  } & React.ComponentProps<typeof List>
> = ({ items, ...rest }) => (
  <List {...rest}>
    {items.map((item) => (
      <NavMenuItem key={item.label} item={item} />
    ))}
  </List>
);

const NavMenuItem: React.FC<{ item: NavItem }> = ({ item }) => {
  const localClasses = useLocalStyles();
  const [childrenOpen, setChildrenOpen] = useState<boolean>(false);

  return (
    <>
      <ListItem
        button
        onClick={() => {
          if (item.children) {
            setChildrenOpen((old) => !old);
          }
          if (item.onClick) {
            item.onClick();
          }
        }}
        {...(item.to
          ? {
              component: UnstyledLink,
              to: item.to,
            }
          : {})}
      >
        <ListItemText>{item.label}</ListItemText>
      </ListItem>

      {item.children && (
        <Collapse in={childrenOpen}>
          <div className={localClasses.nested}>{item.children}</div>
        </Collapse>
      )}
    </>
  );
};

export default NavMenu;
