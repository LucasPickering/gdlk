import React from 'react';
import { Link as RouterLink } from 'react-router-dom';
import { Link as MuiLink } from '@material-ui/core';

type Props = Pick<React.ComponentProps<typeof RouterLink>, 'to'> &
  React.ComponentProps<typeof MuiLink> & { external?: boolean };

/**
 * A component that merges the styles of Material UI's Link with the functionality
 * of React router's Link. If the given target has a protocol, this will assume
 * it's an external link, and use a normal <a> instead of a router link.
 * @param to The link target, either local or external
 */
const Link = ({ to, external, ...rest }: Props): React.ReactElement => {
  const props =
    to.toString().match(/^https?:/) || external
      ? {
          href: to.toString(),
          target: '_blank',
          rel: 'noopener noreferrer',
          ...rest,
        }
      : {
          component: RouterLink,
          to,
          ...rest,
        };

  return <MuiLink {...props} />;
};

Link.defaultProps = { external: false };

export default Link;
