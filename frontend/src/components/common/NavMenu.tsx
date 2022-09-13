import React from "react";
import {
  Collapse,
  List,
  ListItem,
  ListItemText,
  makeStyles,
} from "@material-ui/core";
import UnstyledLink from "./UnstyledLink";
import { useRouteMatch } from "react-router-dom";

const useLocalStyles = makeStyles(({ spacing }) => ({
  nested: {
    paddingLeft: spacing(4),
  },
}));

interface NavItem {
  id: string;
  label: string;
  to?: React.ComponentProps<typeof UnstyledLink>["to"];
  onClick?: () => void;
  children?: React.ReactChild | React.ReactChildren;
}

/**
 * A list of navigation items. Each item can either open a new page/dialog, or
 * it can expand into some sort of subcontent. The subcontent can be any
 * element. To have a nested nav menu, just use another <NavMenu>
 * (with `disablePadding`).
 *
 * Right now, this menu is controlled entirely via routes. Each item gets a
 * route and when that route is active, the item will be selected. You can also
 * have items that don't have an assigned routes, but currently that means they
 * can't show a selected state. If we need that functionality we can extend the
 * component.
 */
const NavMenu: React.FC<
  {
    items: NavItem[];
  } & React.ComponentProps<typeof List>
> = ({ items, ...rest }) => {
  return (
    <List {...rest}>
      {items.map((item) => (
        <NavMenuItem key={item.id} item={item} />
      ))}
    </List>
  );
};

const NavMenuItem: React.FC<{ item: NavItem }> = ({ item }) => {
  const localClasses = useLocalStyles();
  const routeMatch = useRouteMatch(item.to ? item.to.toString() : "");
  const selected = Boolean(routeMatch);

  return (
    <>
      <ListItem
        button
        selected={selected}
        onClick={() => {
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
        <Collapse in={selected}>
          <div className={localClasses.nested}>{item.children}</div>
        </Collapse>
      )}
    </>
  );
};

export default NavMenu;
