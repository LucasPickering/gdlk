import React, { useContext } from 'react';
import { makeStyles } from '@material-ui/core';
import clsx from 'clsx';
import { IdeContext } from 'state/ide';

const LINE_ENDING_RGX = /\r\n|\r|\n/g;

const useLocalStyles = makeStyles(({ palette }) => ({
  codeEditor: {
    display: 'flex',
    overflowY: 'auto',
    backgroundColor: palette.background.default,
  },

  gutter: {
    display: 'flex',
    flexDirection: 'column',
    width: 24,
    padding: 2,
  },
  gutterLineNumber: {
    textAlign: 'right',
    lineHeight: 1.072, // TODO adjust this
  },
  gutterActiveLine: {
    color: palette.primary.main,
  },

  textArea: {
    width: '100%',
    height: '100%',
    resize: 'none',
    overflowY: 'visible',
    backgroundColor: palette.background.default,
    color: palette.text.primary,
  },
}));

/**
 * A GDLK code editor. This is just the text editor plus the gutter, it does
 * not include any controls like building/running.
 */
const CodeEditor: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { machineState, sourceCode, setSourceCode } = useContext(IdeContext);
  const totalLines = (sourceCode.match(LINE_ENDING_RGX)?.length ?? 0) + 1;
  const lineNumbers = Array.from({ length: totalLines }).map((_, i) => i + 1);
  const programCounter = machineState?.programCounter;

  return (
    <div className={clsx(className, localClasses.codeEditor)}>
      <div className={localClasses.gutter}>
        {lineNumbers.map((lineNum) => (
          <span
            key={lineNum}
            className={clsx(localClasses.gutterLineNumber, {
              [localClasses.gutterActiveLine]: programCounter === lineNum - 1,
            })}
          >
            {lineNum}
          </span>
        ))}
      </div>
      <textarea
        className={localClasses.textArea}
        autoFocus
        value={sourceCode}
        onChange={(e) => setSourceCode(e.target.value)}
      />
    </div>
  );
};

export default CodeEditor;
