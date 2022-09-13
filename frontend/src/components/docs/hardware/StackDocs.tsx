import React from "react";
import { Typography } from "@material-ui/core";
import DocsSection from "../DocsSection";
import Link from "../../common/Link";

const StackDocs: React.FC = () => (
  <DocsSection id="stacks" level={3} title="Stacks">
    <Typography>
      Stacks are a high capacity form of value storage. They trade the easy
      accessibility and operability of registers for much higher capacity.
      Operations cannot be performed directly on any values in a stack. The only
      instructions that operate on stacks are{" "}
      <Link to="#instructions--push">PUSH</Link> and{" "}
      <Link to="#instructions--pop">POP</Link>. <code>PUSH</code> puts a new
      value on top of a stack, and <code>POP</code> removes the top value into a
      register. Only the top value of a stack is accessible; all others cannot
      be read until the values above them are popped off.
    </Typography>

    <Typography>
      All stacks are referenced by the naming pattern <code>Sx</code>, where{" "}
      <code>x</code> starts at <code>0</code>. For example, if a machine has 2
      stacks, they will be <code>S0</code> and <code>S1</code>.
    </Typography>

    <Typography>
      Each stack also has a corresponding <Link to="#registers--rsx">RSx</Link>{" "}
      register, which can be used to access the current length of the
      corresponding stack.
    </Typography>

    <DocsSection level={4} title="Capacity">
      <Typography>
        Each stack has a fixed capacity, determined by the hardware spec. Once a
        stack is at capacity, it can no longer be pushed onto. If a push is
        attempted, it will trigger an error.
      </Typography>
    </DocsSection>
  </DocsSection>
);

export default StackDocs;
