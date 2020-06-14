import React, { ReactNode } from 'react';
import { Typography } from '@material-ui/core';
import DocsSection from './DocsSection';

interface Register {
  name: string;
  writable: boolean;
  summary: ReactNode;
}

const REGISTERS: Register[] = [
  {
    name: 'RZR',
    writable: true,
    summary: (
      <>
        Always available. When read from, the value is always 0. When written
        to, the written value is thrown away.
      </>
    ),
  },
  {
    name: 'RLI',
    writable: false,
    summary: (
      <>
        Always available. Holds the current number of values in the{' '}
        <code>INPUT</code> buffer.
      </>
    ),
  },
  {
    name: 'RSx',
    writable: false,
    summary: (
      <>
        Holds the current number of values in the corresponding stack. There is
        one of these for each stack in the machine. Stacks start at{' '}
        <code>S0</code>, so these start at <code>RS0</code>.
      </>
    ),
  },
  {
    name: 'RXx',
    writable: true,
    summary: (
      <>
        General purpose read-write register. Values can be read and written
        freely. Writing overwrites the existing value in the register. The
        number of these available is determined by the hardware spec. Starts at{' '}
        <code>RX0</code>.
      </>
    ),
  },
];

/**
 * The Registers section of the docs.
 */
const RegisterDocs: React.FC = () => {
  return (
    <DocsSection id="registers" level={3} title="Registers">
      <Typography>
        Registers are the basic memory unit of a GDLK machine. Some registers
        are read-only and hold special values, while others are general purpose
        and can be written to. The set of registers available to a machine are
        determined by the hardware specification.
      </Typography>

      <Typography>
        All registers can be read from freely. Reading from a register copies
        its value to the destination, so the register remains untouched. Only
        some registers can be written to. Write behavior varies by register.
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
          {REGISTERS.map(({ name, writable, summary }) => (
            <tr key={name} id={`registers--${name.toLowerCase()}`}>
              <td>
                <code>{name}</code>
              </td>
              <td>{writable ? 'Y' : 'N'}</td>
              <td>{summary}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </DocsSection>
  );
};

export default RegisterDocs;
