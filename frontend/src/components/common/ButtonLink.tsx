import { makeStyles, Button } from '@material-ui/core';
import React from 'react';
import clsx from 'clsx';
import Link from './Link';

const useLocalStyles = makeStyles({
  link: {
    '&:hover': {
      textDecoration: 'none',
    },
  },
});

const ButtonLink: React.FC<
  React.ComponentProps<typeof Button> &
    Pick<React.ComponentProps<typeof Link>, 'to'>
> = ({ className, to, ...rest }) => {
  const localClasses = useLocalStyles();
  return (
    <Link className={clsx(localClasses.link, className)} to={to}>
      <Button {...rest} />
    </Link>
  );
};

export default ButtonLink;
