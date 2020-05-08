import React, { useContext } from 'react';
import { makeStyles } from '@material-ui/core';
import clsx from 'clsx';
import { IdeContext } from 'state/ide';
import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/mode-plain_text';
import 'ace-builds/src-noconflict/theme-terminal';

const useLocalStyles = makeStyles(({ palette }) => ({
  codeEditor: {
    display: 'flex',
    overflowY: 'auto',
    backgroundColor: palette.background.default,
  },
  activeInstruction: {
    position: 'absolute', // Don't remove this!
    backgroundColor: palette.common.white, // TODO more contrast
  },
}));

/**
 * A GDLK code editor. This is just the text editor, it does not include any
 * controls like building/running.
 */
const CodeEditor: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { machineState, sourceCode, setSourceCode } = useContext(IdeContext);
  const programCounter = machineState?.programCounter;

  return (
    <div className={clsx(className, localClasses.codeEditor)}>
      <AceEditor
        name="gdlk-editor"
        mode="plain_text"
        theme="terminal" // TODO use a better theme
        width="100%"
        height="100%"
        value={sourceCode}
        markers={
          // Highlight the NEXT instruction to be executed
          programCounter !== undefined
            ? [
                {
                  startRow: programCounter,
                  startCol: 0,
                  endRow: programCounter,
                  endCol: 50,
                  className: localClasses.activeInstruction,
                  type: 'background',
                },
              ]
            : []
        }
        onChange={setSourceCode}
      />
    </div>
  );
};

export default CodeEditor;
