import { Typography } from "@mui/material";
import React from "react";
import Link from "./common/Link";

const AboutPage: React.FC = () => (
  <div>
    <Typography variant="body2">
      Created by <Link to="https://github.com/JRMurr">John Murray</Link> and{" "}
      <Link to="https://lucaspickering.me">Lucas Pickering</Link>
    </Typography>
    <Link to="https://github.com/LucasPickering/gdlk">GitHub</Link>
  </div>
);

export default AboutPage;
