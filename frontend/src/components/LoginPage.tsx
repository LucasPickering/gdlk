import React from 'react';
import { Button, Grid } from '@material-ui/core';
import Link from 'components/common/Link';

// TODO: get providers from api
const LoginPage: React.FC = () => {
  return (
    <Grid container justify="center">
      <Grid item md={4} sm={8} xs={12}>
        <Button
          color="primary"
          component={Link}
          to="/api/oidc/redirect"
          external
        >
          Login with Google
        </Button>
      </Grid>
    </Grid>
  );
};

export default LoginPage;
