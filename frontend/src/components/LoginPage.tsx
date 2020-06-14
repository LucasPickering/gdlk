import React from 'react';
import { Button, Grid } from '@material-ui/core';
import UnstyledLink from './common/UnstyledLink';

// TODO: get providers from api
const LoginPage: React.FC = () => {
  return (
    <Grid container justify="center">
      <Grid item md={4} sm={8} xs={12}>
        <Button
          color="primary"
          component={UnstyledLink}
          to="/api/oidc/google/redirect"
        >
          Login with Google
        </Button>
      </Grid>
    </Grid>
  );
};

export default LoginPage;
