import React, { ReactNode, useContext } from "react";
import { Typography } from "@mui/material";
import DocsSection from "../DocsSection";
import { DocsContext, DocsContextType } from "@root/state/docs";

interface Register {
  name: string;
  writable: boolean;
  summary: ReactNode;
  /**
   * Allows us to dynamically hide entries based on capabilities
   */
  isVisible?: (context: DocsContextType) => boolean;
}

const REGISTERS: Register[] = [
  {
    name: "RXx",
    writable: true,
    summary: (
      <>
        Register eXtension x: General-purpose read/write register. Values can be
        read and written freely. Writing overwrites the existing value in the
        register. The number of <code>RXx</code> registers available depends on
        how many are installed in your GDLKx PC. Register IDs start at 0 and
        increment from there. For example, a PC with two general-purpose
        registers will have <code>RX0</code> and <code>RX1</code>.
      </>
    ),
  },
  {
    name: "RSx",
    writable: false,
    summary: (
      <>
        Register Stack x: Holds the current number of values in the
        corresponding stack. There is one of these for each stack in the
        machine. Stacks start at <code>S0</code>, so stack length registers
        start at <code>RS0</code>.
      </>
    ),
    isVisible: (context) => context.showStacks,
  },
  {
    name: "RLI",
    writable: false,
    summary: (
      <>
        Register Length Input: Holds the current number of values in the{" "}
        <code>INPUT</code> buffer.
      </>
    ),
  },
  {
    name: "RZR",
    writable: true,
    summary: (
      <>
        Register ZeRo: When read from, the value is always 0. When written to,
        the written value is thrown away.
      </>
    ),
  },
];

/**
 * The Registers section of the docs.
 */
const RegisterDocs: React.FC = () => {
  const context = useContext(DocsContext);
  const visibleRegisters = REGISTERS.filter(
    ({ isVisible }) => isVisible?.(context) ?? true
  );

  return (
    <DocsSection id="registers" level={3} title="Registers">
      <Typography>
        Registers are the basic memory unit of a GDLKx machine. Some registers
        are read-only and hold special values, while others are general-purpose
        and can be written to by your GDLK programs.
      </Typography>

      <Typography>
        All registers can be read from freely, simply by using them in a read
        position of an instruction. Read operations do not modify the value in
        the register. Only some registers can be written to. Write behavior
        varies by register.
      </Typography>

      <table>
        <thead>
          <tr>
            <th>Register</th>
            <th>Writable?</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          {visibleRegisters.map(({ name, writable, summary }) => (
            <tr key={name} id={`registers--${name.toLowerCase()}`}>
              <td>
                <code>{name}</code>
              </td>
              <td>{writable ? "Yes" : "No"}</td>
              <td>{summary}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </DocsSection>
  );
};

export default RegisterDocs;
