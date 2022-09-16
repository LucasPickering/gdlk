import React from "react";
import { Typography } from "@mui/material";
import DocsSection from "../DocsSection";
import InstructionDocs from "./InstructionDocs";

const MIN_LANG_VALUE = -2147483648;
const MAX_LANG_VALUE = 2147483647;

const LanguageDocs: React.FC = () => (
  <DocsSection
    id="language-reference"
    level={2}
    title="GDLK Language Reference"
  >
    <DocsSection id="values" level={3} title="Values">
      <Typography>
        All GDLK values are 32-bit signed integers, meaning they fall in the
        range{" "}
        <code>
          [{MIN_LANG_VALUE}, {MAX_LANG_VALUE}]
        </code>
        . Encoding systems can be built on top of these values, but all hardware
        operations are performed on these integers.
      </Typography>

      <Typography id="values--overflow-and-underflow" variant="h4">
        Overflow & Underflow
      </Typography>
      <Typography>
        When an arithmetic instruction causes a value to go above the max or
        below the min, the value wraps around. For example,{" "}
        <code>
          {MAX_LANG_VALUE} + 1 = {MIN_LANG_VALUE}
        </code>
        , and{" "}
        <code>
          {MIN_LANG_VALUE} - 1 = {MAX_LANG_VALUE}
        </code>
        .
      </Typography>
    </DocsSection>

    <DocsSection id="labels" level={3} title="Labels">
      A label references a particular position in code, which can be used as the
      target of a jump instruction. For example:
      <pre>
        <code>
          {`; An infinite loop that repeatedly writes 0 to output
LOOP:
  WRITE 0
  JMP LOOP`}
        </code>
      </pre>
      Typically, the code after a label is indented for readability purposes,
      but this is simply a convention and <em>not required</em>.
    </DocsSection>

    <InstructionDocs />
  </DocsSection>
);

export default LanguageDocs;
