import React from "react";
import { Grid } from "@mui/material";
import { puzzles } from "@root/data/puzzles";
import NavMenu from "./common/NavMenu";
import PuzzleList from "./puzzle/PuzzleList";
import { Outlet, useParams } from "react-router-dom";

const HomePage: React.FC = () => {
  const { puzzleSlug } = useParams();

  return (
    <Grid container>
      <Grid item md={4} sm={8} xs={12}>
        <NavMenu
          items={[
            {
              id: "puzzles",
              label: "Puzzles",
              to: "/puzzles",
              children: (
                <PuzzleList
                  puzzles={Object.values(puzzles)}
                  selectedPuzzle={puzzleSlug}
                  link
                />
              ),
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
