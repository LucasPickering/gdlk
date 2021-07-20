import React from 'react';
import { makeStyles, Paper, Typography } from '@material-ui/core';
import Link from '@root/components/common/Link';
import RegisterDocs from './RegisterDocs';
import InstructionDocs from './InstructionDocs';
import DocsSection from './DocsSection';

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  docs: {
    padding: spacing(2),

    '& code': {
      color: palette.text.secondary,
    },
    '& pre': {
      backgroundColor: palette.background.default,
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

/**
 * All of the language docs content. This content is entirely static.
 */
const DocsPage: React.FC = () => {
  const localClasses = useLocalStyles();

  return (
    <Paper className={localClasses.docs}>
      <Typography variant="h1">GDLK Documentation</Typography>

      <DocsSection id="introduction" level={2} title="Introduction">
        <Typography>
          GDLK programs execute on a certain machine, whose capabilities are
          determined by a hardware specification. They also execute under
          certain input/output conditions, known as a program specification. The
          goal of a GDLK program is to read values from the input and transform
          them in order to generate the proper output.
        </Typography>

        <Typography>
          The hardware components of a machine are the{' '}
          <Link to="#input-and-output">input & output</Link>,{' '}
          <Link to="#registers">registers</Link>, and{' '}
          <Link to="#stacks">stacks</Link>. A program consists of a series of{' '}
          <Link to="#instructions">instructions</Link>, can be used to apply a
          variety of data operations.
        </Typography>
      </DocsSection>

      <DocsSection id="language-reference" level={2} title="Language Reference">
        <DocsSection id="values" level={3} title="Values">
          <Typography>
            All GDLK values are 32-bit signed integers, meaning they fall in the
            range <code>[-2147483648, 2147483647]</code>. Encoding systems can
            be built on top of these values, but all hardware operations are
            performed on these integers.
          </Typography>

          <Typography id="values--overflow-and-underflow" variant="h4">
            Overflow & Underflow
          </Typography>
          <Typography>
            When an arithmetic instruction causes a value to go above the max or
            below the min, the value wraps around. For example,{' '}
            <code>32767 + 1 = -32768</code>, and <code>-32768 - 1 = 32767</code>
            .
          </Typography>
        </DocsSection>

        <RegisterDocs />
        {/* Hidden for playtesting simple puzzles */}
        {/* <StackDocs /> */}

        <DocsSection id="input-and-output" level={3} title="Input & Output">
          <Typography>
            Programs interact with outside values via input and output. The
            program specification defines what values a program takes as input.
            It also defines which values it expects as output. The purpose of
            each program then, is to consume all of the input, and generate the
            proper output.
          </Typography>

          <DocsSection level={4} title="Reading from Input">
            <ul>
              <li>
                The <Link to="#instructions--read">READ</Link> instruction is
                used to read from <code>INPUT</code>.
              </li>
              <li>
                <code>INPUT</code> is read-only. Values cannot be written to it.
              </li>
              <li>
                Values are read from the front of <code>INPUT</code>. Once a
                value has been read, it is removed from the buffer.
              </li>
              <li>
                Once <code>INPUT</code> is empty, subsequent reads will cause an
                error.
              </li>
              <li>
                The <Link to="#registers--rli">RLI</Link> register can be used
                to check how many values are left in <code>INPUT</code>.
              </li>
            </ul>
          </DocsSection>

          <DocsSection level={4} title="Writing to Output">
            <ul>
              <li>
                The <Link to="#instructions--write">WRITE</Link> instruction is
                used to write to <code>OUTPUT</code>.
              </li>
              <li>
                <code>OUTPUT</code> is write-only. Values in the buffer cannot
                be read or modified.
              </li>
              <li>
                Values are written to the back of the <code>OUTPUT</code>{' '}
                buffer.
              </li>
              <li>
                <code>OUTPUT</code> has no capacity limit.
              </li>
              <li>
                There is no way to read the current number of values in the
                buffer.
              </li>
            </ul>
          </DocsSection>
        </DocsSection>

        <InstructionDocs />
      </DocsSection>
    </Paper>
  );
};

export default DocsPage;
