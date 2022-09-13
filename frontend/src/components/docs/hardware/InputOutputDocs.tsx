import { Typography } from "@material-ui/core";
import React from "react";
import Link from "../../common/Link";
import DocsSection from "../DocsSection";

const InputOutputDocs: React.FC = () => (
  <DocsSection id="input-and-output" level={3} title="Input & Output">
    <Typography>
      GDLKx PCs interact with external data via their input and output buffers.
      Each buffer represents a one-way stream of values, meaning the buffer is
      either read-only (in the case of input) or write-only (in the case of
      output).
    </Typography>

    <DocsSection level={4} title="Reading from Input">
      <ul>
        <li>
          The <Link to="#instructions--read">READ</Link> instruction is used to
          read from <code>INPUT</code>.
        </li>
        <li>
          <code>INPUT</code> is read-only. Values cannot be written to it.
        </li>
        <li>
          Values are read from the front of <code>INPUT</code>. Once a value has
          been read, it is removed from the buffer.
        </li>
        <li>
          Once <code>INPUT</code> is empty, subsequent reads will cause an
          error.
        </li>
        <li>
          The <Link to="#registers--rli">RLI</Link> register can be used to
          check how many values are left in <code>INPUT</code>.
        </li>
      </ul>
    </DocsSection>

    <DocsSection level={4} title="Writing to Output">
      <ul>
        <li>
          The <Link to="#instructions--write">WRITE</Link> instruction is used
          to write to <code>OUTPUT</code>.
        </li>
        <li>
          <code>OUTPUT</code> is write-only. Values in the buffer cannot be read
          or modified.
        </li>
        <li>
          Values are written to the back of the <code>OUTPUT</code> buffer.
        </li>
        <li>
          <code>OUTPUT</code> has no capacity limit.
        </li>
        <li>
          There is no way to read the current number of values in the buffer.
        </li>
      </ul>
    </DocsSection>
  </DocsSection>
);

export default InputOutputDocs;
