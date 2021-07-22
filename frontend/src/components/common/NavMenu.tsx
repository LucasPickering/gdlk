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
  id: string;
  label: string;
  to?: React.ComponentProps<typeof UnstyledLink>['to'];
  onClick?: () => void;
  children?: React.ReactChild | React.ReactChildren;
}

/**
 * A list of navigation items. Each item can either open a new page/dialog, or
 * it can expand into some sort of subcontent. The subcontent can be any
 * element. To have a nested nav menu, just use another <NavMenu>
 * (with `disablePadding`)
 */
const NavMenu: React.FC<
  {
    items: NavItem[];
    initialExpandedItem?: string;
  } & React.ComponentProps<typeof List>
> = ({ items, initialExpandedItem, ...rest }) => {
  const localClasses = useLocalStyles();
  const [expandedItem, setExpandedItem] = useState<string | undefined>(
    initialExpandedItem
  );

  return (
    <List {...rest}>
      {items.map((item) => (
        <React.Fragment key={item.id}>
          <ListItem
            button
            selected={item.id === expandedItem}
            onClick={() => {
              if (item.children) {
                setExpandedItem((old) =>
                  old === item.id ? undefined : item.id
                );
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
            <Collapse in={expandedItem === item.id}>
              <div className={localClasses.nested}>{item.children}</div>
            </Collapse>
          )}
        </React.Fragment>
      ))}
    </List>
  );
};

export default NavMenu;
