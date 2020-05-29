import React, { ReactNode } from 'react';
import {
  makeStyles,
  Card,
  CardContent,
  Typography,
  CardHeader,
} from '@material-ui/core';
import Link from 'components/common/Link';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  docs: {
    '& code': {
      color: palette.text.secondary,
    },
    '& pre': {
      backgroundColor: palette.background.default,
      padding: spacing(1),
    },

    '& section + section': {
      marginTop: spacing(1),
    },

    // Table styles
    '& table': {
      border: `2px solid ${palette.divider}`,
      borderCollapse: 'collapse',
    },
    '& th, td': {
      border: `1px solid ${palette.divider}`,
      padding: spacing(0.5),
    },
  },
}));

type ArgType = 'VAL' | 'REG' | 'STACK';

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
        The stored value is determined by these rules:
        <table>
          <thead>
            <tr>
              <th>Condition</th>
              <th>Output</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>
                <code>first &lt; second</code>
              </td>
              <td>
                <code>-1</code>
              </td>
            </tr>
            <tr>
              <td>
                <code>first = second</code>
              </td>
              <td>
                <code>0</code>
              </td>
            </tr>
            <tr>
              <td>
                <code>first &gt; second</code>
              </td>
              <td>
                <code>1</code>
              </td>
            </tr>
          </tbody>
        </table>
      </>
    ),
    args: ['REG', 'VAL', 'VAL'],
    examples: [
      'CMP RX0 10 11 ; RX0 now holds -1',
      'CMP RX0 11 11 ; RX0 now holds 0',
      'CMP RX0 11 10 ; RX0 now holds 1',
    ],
  },
  {
    name: 'PUSH',
    summary: 'Push a value onto the top of a stack.',
    moreInfo: <>The source value is not modified.</>,
    args: ['VAL', 'STACK'],
    errorCases: [
      <>
        Pushing to a stack that is already full causes a runtime error.{' '}
        <Link to="#stacks--capacity">More information on stack capacity</Link>.
      </>,
    ],
    examples: [
      'PUSH 3 S0   ; Push 3 onto the top of S0',
      'PUSH RX0 S0 ; Push the value in RX0 onto the top of S0',
    ],
  },
  {
    name: 'POP',
    summary: 'Pop a value off the top of a stack into a register.',
    args: ['STACK', 'REG'],
    errorCases: [<>Popping from an empty stack causes a runtime error.</>],
    examples: ['POP S0 RX0 ; Move the top value of S0 into RX0'],
  },
];

const DocsPage: React.FC = () => {
  // This HTML is rendered directly from our own docs file, so this is safe
  const localClasses = useLocalStyles();
  return (
    <Card className={localClasses.docs}>
      <CardHeader
        title={<Typography variant="h1">GDLK Documentation</Typography>}
      />
      <CardContent>
        <section>
          <Typography variant="h2">Language Reference</Typography>

          <section>
            <Typography variant="h3">Values</Typography>
            <Typography>
              All GDLK values are integers in the range{' '}
              <code>[-32768, 32767]</code>. Encoding systems can be built on top
              of these values, but all hardware operations are performed on
              these integers.
            </Typography>

            <Typography id="values--overflow-and-underflow" variant="h4">
              Overflow & Underflow
            </Typography>
            <Typography>
              When an arithmetic instruction causes a value to go above the max
              to below the min, the value wraps around. For example,{' '}
              <code>32767 + 1 = -32768</code>, and{' '}
              <code>-32768 - 1 = 32767</code>.
            </Typography>
          </section>

          <section>
            <Typography variant="h3">Input & Output</Typography>
            <Typography>TODO</Typography>
          </section>

          <section>
            <Typography id="instructions" variant="h3">
              Instructions
            </Typography>
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
                      <code>{name}</code>
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
                <section key={name}>
                  <Typography variant="h4">{name}</Typography>

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
                  <pre>
                    <code>{examples.join('\n')}</code>
                  </pre>
                </section>
              )
            )}
          </section>

          <section>
            <Typography variant="h3">Registers</Typography>
            <Typography>TODO</Typography>
          </section>

          <section>
            <Typography variant="h3">Stacks</Typography>
            <Typography>TODO</Typography>

            <section>
              <Typography id="stacks--capacity" variant="h4">
                Capacity
              </Typography>
            </section>
            <Typography>TODO</Typography>
          </section>
        </section>
      </CardContent>
    </Card>
  );
};

export default DocsPage;
