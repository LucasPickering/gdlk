import React, { ReactNode } from 'react';
import { Typography } from '@material-ui/core';
import Link from '@root/components/common/Link';
import DocsSection from './DocsSection';

type ArgType = 'VAL' | 'REG' | 'STACK' | 'LABEL';

interface InstructionReference {
  name: string;
  args: ArgType[];
  summary: ReactNode;
  moreInfo?: ReactNode;
  errorCases?: ReactNode[];
  notes?: ReactNode[];
  examples: string[];
}

const INSTRUCTIONS: InstructionReference[] = [
  {
    name: 'READ',
    args: ['REG'],
    summary: (
      <>
        Read the next value from <code>INPUT</code> and store it in a register.
      </>
    ),
    moreInfo: (
      <>
        The value is removed from <code>INPUT</code>, and cannot be returned.
      </>
    ),
    errorCases: [
      <>
        Reading while <code>INPUT</code> is empty causes a runtime error.
      </>,
    ],
    examples: ['READ RX0 ; Move the first value in INPUT into RX0'],
  },
  {
    name: 'WRITE',
    args: ['VAL'],
    summary: (
      <>
        Write a value to <code>OUTPUT</code>.
      </>
    ),
    moreInfo: (
      <>
        Once a value has been written to <code>OUTPUT</code>, it cannot be
        changed, moved, or removed. The source value is not modified.
      </>
    ),
    examples: [
      'WRITE 3   ; Write 3 to OUTPUT',
      'WRITE RX0 ; Write the value in RX0 to OUTPUT',
    ],
  },
  {
    name: 'SET',
    summary: 'Set a register to a value.',
    args: ['REG', 'VAL'],
    examples: [
      'SET RX0 3   ; RX0 now holds the value 3',
      'SET RX0 RX1 ; RX0 now holds whatever value is in RX1',
    ],
  },
  {
    name: 'ADD',
    summary: 'Add a value to a register.',
    moreInfo: (
      <>
        The result is stored in the register. See{' '}
        <Link to="#values--overflow-and-underflow">
          arithmetic overflow and underflow
        </Link>
        .
      </>
    ),
    args: ['REG', 'VAL'],
    examples: [
      'ADD RX0 3   ; Add 3 to whatever value is in RX0',
      'ADD RX0 RX1 ; Add the value in RX1 to RX0',
    ],
  },
  {
    name: 'SUB',
    summary: 'Subtract a value from a register.',
    moreInfo: (
      <>
        The result is stored in the register. See{' '}
        <Link to="#values--overflow-and-underflow">
          arithmetic overflow and underflow
        </Link>
        .
      </>
    ),
    args: ['REG', 'VAL'],
    examples: [
      'SUB RX0 3   ; Subtract 3 from whatever value is in RX0',
      'SUB RX0 RX1 ; Subtract the value in RX1 from RX0',
    ],
  },
  {
    name: 'MUL',
    summary: 'Multiply a register by a value.',
    moreInfo: (
      <>
        The result is stored in the register. See{' '}
        <Link to="#values--overflow-and-underflow">
          arithmetic overflow and underflow
        </Link>
        .
      </>
    ),
    args: ['REG', 'VAL'],
    examples: [
      'MUL RX0 3   ; Multiply the value in RX0 by 3',
      'MUL RX0 RX1 ; Multiply the value in RX0 by the value in RX1',
    ],
  },
  {
    name: 'DIV',
    summary: 'Divide a register by a value.',
    moreInfo: (
      <>
        The remainder is thrown away, i.e. the result is always rounded down.
        The result is stored in the register.
      </>
    ),
    args: ['REG', 'VAL'],
    errorCases: ['Dividing by zero causes a runtime error.'],
    examples: [
      'DIV RX0 3   ; Divide the value in RX0 by 3',
      'DIV RX0 RX1 ; Divide the value in RX0 by the value in RX1',
    ],
  },
  {
    name: 'CMP',
    summary: 'Compare two values, and put the output into a register.',
    moreInfo: (
      <>
        If <code>first &lt; second</code>, the result is <code>-1</code>. If{' '}
        <code>first = second</code>, the result is <code>0</code>. If{' '}
        <code>first &gt; second</code>, the result is <code>1</code>.
      </>
    ),
    args: ['REG', 'VAL', 'VAL'],
    examples: [
      'CMP RX0 10 11 ; RX0 now holds -1',
      'CMP RX0 11 11 ; RX0 now holds 0',
      'CMP RX0 11 10 ; RX0 now holds 1',
    ],
  },
  // Commented out for playtesting simple puzzles
  // {
  //   name: 'PUSH',
  //   summary: 'Push a value onto the top of a stack.',
  //   moreInfo: <>The source value is not modified.</>,
  //   args: ['VAL', 'STACK'],
  //   errorCases: [
  //     <>
  //       Pushing to a stack that is already full causes a runtime error.{' '}
  //       <Link to="#stacks--capacity">More information on stack capacity</Link>.
  //     </>,
  //   ],
  //   examples: [
  //     'PUSH 3 S0   ; Push 3 onto the top of S0',
  //     'PUSH RX0 S0 ; Push the value in RX0 onto the top of S0',
  //   ],
  // },
  // {
  //   name: 'POP',
  //   summary: 'Pop a value off the top of a stack into a register.',
  //   args: ['STACK', 'REG'],
  //   errorCases: [<>Popping from an empty stack causes a runtime error.</>],
  //   examples: ['POP S0 RX0 ; Move the top value of S0 into RX0'],
  // },
  {
    name: 'JMP',
    summary: 'Jump to a label, unconditionally.',
    args: ['LABEL'],
    examples: [
      'JMP END\nREAD RX0 ; This instruction will be skipped\nEND:',
      'LOOP:\nADD RX0 1\nJMP LOOP ; Infinite loop',
    ],
  },
  {
    name: 'JEZ',
    summary: (
      <>
        Jump to a label if the value is equal to <code>0</code>.
      </>
    ),
    args: ['VAL', 'LABEL'],
    examples: [
      'SET RX0 0\nJEZ END\nREAD RX0 ; This instruction will be skipped\nEND:',
      'SET RX0 1\nJEZ END\nREAD RX0 ; This instruction will NOT be skipped\nEND:',
    ],
  },
  {
    name: 'JNZ',
    summary: (
      <>
        Jump to a label if the value is NOT equal to <code>0</code>.
      </>
    ),
    args: ['VAL', 'LABEL'],
    examples: [
      'SET RX0 1\nJNZ END\nREAD RX0 ; This instruction will be skipped\nEND:',
      'SET RX0 0\nJNZ END\nREAD RX0 ; This instruction will NOT be skipped\nEND:',
    ],
  },
  {
    name: 'JGZ',
    summary: (
      <>
        Jump to a label if the value is greater than <code>0</code>.
      </>
    ),
    args: ['VAL', 'LABEL'],
    examples: [
      'SET RX0 1\nJGZ END\nREAD RX0 ; This instruction will be skipped\nEND:',
      'SET RX0 -1\nJGZ END\nREAD RX0 ; This instruction will NOT be skipped\nEND:',
    ],
  },
  {
    name: 'JLZ',
    summary: (
      <>
        Jump to a label if the value is less than <code>0</code>.
      </>
    ),
    args: ['VAL', 'LABEL'],
    examples: [
      'SET RX0 -1\nJLZ END\nREAD RX0 ; This instruction will be skipped\nEND:',
      'SET RX0 1\nJLZ END\nREAD RX0 ; This instruction will NOT be skipped\nEND:',
    ],
  },
];

/**
 * The Instructions section of the docs.
 */
const InstructionDocs: React.FC = () => {
  return (
    <DocsSection id="instructions" level={3} title="Instructions">
      <table>
        <thead>
          <tr>
            <th>Instruction</th>
            <th>Arguments</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          {INSTRUCTIONS.map(({ name, args, summary }) => (
            <tr key={name}>
              <td>
                <Link to={`#instructions--${name.toLowerCase()}`}>{name}</Link>
              </td>
              <td>
                <code>{args.map((arg) => `<${arg}>`).join(' ')}</code>
              </td>
              <td>{summary}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {INSTRUCTIONS.map(
        ({ name, args, summary, moreInfo, errorCases, examples }) => (
          <DocsSection
            key={name}
            id={`instructions--${name.toLowerCase()}`}
            level={4}
            title={name}
          >
            <code>
              {name} {args.map((arg) => `<${arg}>`).join(' ')}
            </code>

            <Typography>
              {summary} {moreInfo}
            </Typography>

            {errorCases && (
              <>
                <Typography variant="h5">Errors</Typography>
                <ul>
                  {errorCases.map((errorCase, i) => (
                    <li key={i}>{errorCase}</li>
                  ))}
                </ul>
              </>
            )}

            <Typography variant="h5">Examples</Typography>
            {examples.map((example, i) => (
              <pre key={i}>
                <code>{example}</code>
              </pre>
            ))}
          </DocsSection>
        )
      )}
    </DocsSection>
  );
};

export default InstructionDocs;
