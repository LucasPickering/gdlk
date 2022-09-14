import { AppBar, Box, Toolbar, Typography } from "@material-ui/core";
import React from "react";
import { useRecoilValue } from "recoil";
import { currencyState } from "@root/state/user";
import { formatCurrency } from "@root/util/format";
import Clock from "./Clock";

const HeaderBar: React.FC = () => {
  const currency = useRecoilValue(currencyState);

  return (
    <AppBar position="static">
      <Toolbar variant="dense">
        <Typography variant="h2" component="h1">
          GDLK_OS
        </Typography>

        <Box display="flex" flexDirection="row" marginLeft="auto">
          <Typography>{formatCurrency(currency)}</Typography>
          <Clock />
        </Box>
      </Toolbar>
    </AppBar>
  );
};

export default HeaderBar;
