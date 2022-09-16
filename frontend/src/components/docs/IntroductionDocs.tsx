import React from "react";
import { Typography } from "@mui/material";
import DocsSection from "./DocsSection";

const IntroductionDocs: React.FC = () => (
  <DocsSection id="introduction" level={2} title="Introduction">
    <Typography>
      GDLK is a state-of-the-art general-purpose programming language designed
      to run on the GDLKx family of personal computers.
    </Typography>

    <Typography>
      All values in GDLK are integers. The basic memory unit of GDLK is the{" "}
      <em>register</em>, and the basic operational unit is the{" "}
      <em>instruction</em>. Each program operates on an input stream of values
      and produces a stream of output values.
    </Typography>

    <Typography>
      Here is an example of a simple GDLK program, which reads a single value
      from input and writes it to output, using the register <code>RX0</code> as
      an intermediate data store.
    </Typography>
    <pre>
      <code>{`READ RX0 ; Read the top value from the input buffer into the register RX0
WRITE RX0 ; Write the value from RX0 into the output buffer`}</code>
    </pre>
  </DocsSection>
);

export default IntroductionDocs;
