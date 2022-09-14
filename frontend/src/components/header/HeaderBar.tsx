import { AppBar, Stack, Toolbar, Typography } from "@mui/material";
import React from "react";
import Clock from "./Clock";
import Wallet from "./Wallet";

const HeaderBar: React.FC = () => (
  <AppBar position="static">
    <Toolbar variant="dense">
      <Typography variant="h2" component="h1">
        GDLK_OS
      </Typography>

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
