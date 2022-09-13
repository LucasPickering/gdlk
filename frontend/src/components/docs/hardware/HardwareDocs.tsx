import React, { useContext } from "react";
import { Typography } from "@material-ui/core";
import DocsSection from "../DocsSection";
import InputOutputDocs from "./InputOutputDocs";
import RegisterDocs from "./RegisterDocs";
import Link from "../../common/Link";
import { DocsContext } from "@root/state/docs";
import StackDocs from "./StackDocs";

const HardwareDocs: React.FC = () => {
  const { showStacks } = useContext(DocsContext);

  return (
    <DocsSection
      id="hardware-reference"
      level={2}
      title="GDLKx Hardware Reference"
    >
      <Typography component="div">
        The basic hardware components of a GDLKx machine are:
        <ul>
          <li>
            <Link to="#registers">Registers</Link>
          </li>
          <li>
            <Link to="#input-and-output">Input & Output Buffers</Link>
          </li>
          {showStacks && (
            <li>
              <Link to="#stacks">Stacks</Link>
            </li>
          )}
        </ul>
      </Typography>

      <RegisterDocs />
      <InputOutputDocs />
      {showStacks && <StackDocs />}
    </DocsSection>
  );
};

export default HardwareDocs;
