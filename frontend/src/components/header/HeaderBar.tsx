import { AppBar, Stack, Toolbar, Typography } from "@mui/material";
import React from "react";
import Link from "../common/Link";
import Clock from "./Clock";
import Wallet from "./Wallet";

const HeaderBar: React.FC = () => (
  <AppBar position="static">
    <Toolbar variant="dense">
      <Link to="/">
        <Typography variant="h2" component="h1">
          GDLK_OS
        </Typography>
      </Link>

      <Typography variant="h4" marginLeft="auto">
        <Stack direction="row" spacing={2}>
          <Wallet />
          <Clock />
        </Stack>
      </Typography>
    </Toolbar>
  </AppBar>
);

export default HeaderBar;
