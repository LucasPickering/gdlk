import React from 'react';
import { makeStyles, Typography } from '@material-ui/core';
import Link from '@root/components/common/Link';
import RegisterDocs from './RegisterDocs';
import InstructionDocs from './InstructionDocs';
import DocsSection from './DocsSection';
import InputOutputDocs from './InputOutputDocs';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  docs: {
    padding: spacing(2),

    '& code': {
      color: palette.text.secondary,
    },
    '& pre': {
      backgroundColor: palette.background.paper,
      padding: spacing(1),
    },

    // Text styles
    '& h2': {
      marginTop: spacing(4),
    },
    '& h3': {
      marginTop: spacing(3),
    },
    '& h4': {
      marginTop: spacing(2),
    },
    '& h5, p': {
      marginTop: spacing(1),
    },

    '& ul': {
      margin: 0,
    },

    // Table styles
    '& table': {
      marginTop: spacing(2),
      border: `2px solid ${palette.divider}`,
      borderCollapse: 'collapse',
    },
    '& th, td': {
      border: `1px solid ${palette.divider}`,
      padding: spacing(0.5),
    },
  },
}));

const MIN_LANG_VALUE = -2147483648;
const MAX_LANG_VALUE = 2147483647;

/**
 * All of the language docs content. This content is entirely static.
 */
const DocsPage: React.FC = () => {
  const localClasses = useLocalStyles();

  // This content is written from within the GDLK canon, i.e. from the
  // perspective that GDLK is a real language and the reader owns a GDLKx PC.
  return (
    <div className={localClasses.docs}>
      <Typography variant="h1">GDLK Documentation</Typography>

      <DocsSection id="introduction" level={2} title="Introduction">
        <Typography>
          GDLK is a state-of-the-art general-purpose programming language
          designed to run on the GDLKx family of personal computers.
        </Typography>

        <Typography>
          Here is an example of a simple GDLK program, which reads a single
          value from input and writes it to output, using the register{' '}
          <code>RX0</code> as an intermediate data store.
        </Typography>
        <pre>
          <code>{`READ RX0 ; Read the top value from the input buffer into the register RX0
WRITE RX0 ; Write the value from RX0 into the output buffer`}</code>
        </pre>
      </DocsSection>

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
            {/* Hidden for playtesting simple puzzles */}
            {/* <li>
              <Link to="#stacks">Stacks</Link>
            </li> */}
          </ul>
        </Typography>

        <RegisterDocs />
        <InputOutputDocs />
        {/* Hidden for playtesting simple puzzles */}
        {/* <StackDocs /> */}
      </DocsSection>

      <DocsSection
        id="language-reference"
        level={2}
        title="GDLK Language Reference"
      >
        <DocsSection id="values" level={3} title="Values">
          <Typography>
            All GDLK values are 32-bit signed integers, meaning they fall in the
            range{' '}
            <code>
              [{MIN_LANG_VALUE}, {MAX_LANG_VALUE}]
            </code>
            . Encoding systems can be built on top of these values, but all
            hardware operations are performed on these integers.
          </Typography>

          <Typography id="values--overflow-and-underflow" variant="h4">
            Overflow & Underflow
          </Typography>
          <Typography>
            When an arithmetic instruction causes a value to go above the max or
            below the min, the value wraps around. For example,{' '}
            <code>
              {MAX_LANG_VALUE} + 1 = {MIN_LANG_VALUE}
            </code>
            , and{' '}
            <code>
              {MIN_LANG_VALUE} - 1 = {MAX_LANG_VALUE}
            </code>
            .
          </Typography>
        </DocsSection>

        <InstructionDocs />
      </DocsSection>
    </div>
  );
};

export default DocsPage;
