import React from "react";
import { Grid } from "@mui/material";
import NavMenu from "./common/NavMenu";
import { Outlet } from "react-router-dom";

const HomePage: React.FC = () => {
  return (
    <Grid container spacing={2}>
      <Grid item md={4} sm={8} xs={12}>
        <NavMenu
          items={[
            {
              id: "puzzles",
              label: "Puzzles",
              to: "/puzzles",
            },
            {
              id: "hardware",
              label: "Hardware",
              to: "/hardware",
            },
            {
              id: "docs",
              label: "GDLK Reference Guide",
              to: "/docs",
            },
            {
              id: "about",
              label: "About",
              to: "/about",
            },
          ]}
        />
      </Grid>

      <Grid item md={8} sm={12}>
        <Outlet />
      </Grid>
    </Grid>
  );
};

export default HomePage;
