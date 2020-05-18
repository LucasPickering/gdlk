import React, { useContext } from 'react';
import { makeStyles } from '@material-ui/core';
import clsx from 'clsx';
import { IdeContext, gdlkSpanToAce } from 'state/ide';
import AceEditor, { IAnnotation, IMarker } from 'react-ace';
import 'ace-builds/src-noconflict/mode-plain_text';
import 'ace-builds/src-noconflict/theme-terminal';

const useLocalStyles = makeStyles(({ palette }) => ({
  codeEditor: {
    display: 'flex',
    overflowY: 'auto',
    backgroundColor: palette.background.default,
  },
  errorSpan: {
    position: 'absolute',
    backgroundColor: palette.error.dark,
  },
  activeInstruction: {
    position: 'absolute', // Don't remove this!
    backgroundColor: palette.grey[600],
  },
}));

/**
 * A GDLK code editor. This is just the text editor, it does not include any
 * controls like building/running.
 */
const CodeEditor: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { compiledState, sourceCode, setSourceCode } = useContext(IdeContext);

  const markers: IMarker[] = [];
  const annotations: IAnnotation[] = [];

  switch (compiledState?.type) {
    case 'compiled':
      {
        // Highlight the NEXT instruction to be executed
        const {
          machineState: { programCounter, runtimeError },
        } = compiledState;
        const nextInstruction = compiledState.instructions[programCounter];

        if (nextInstruction) {
          markers.push({
            ...gdlkSpanToAce(nextInstruction.span),
            className: localClasses.activeInstruction,
            type: 'fullLine',
          });
        }

        if (runtimeError) {
          const aceSpan = gdlkSpanToAce(runtimeError.span);
          markers.push({
            ...aceSpan,
            className: localClasses.activeInstruction,
            type: 'fullLine',
          });
          annotations.push({
            row: aceSpan.startRow,
            column: aceSpan.startCol,
            text: runtimeError.text,
            type: 'error',
          });
        }
      }
      break;

    case 'error':
      compiledState.errors.forEach((error) => {
        const aceSpan = gdlkSpanToAce(error.span);
        markers.push({
          ...aceSpan,
          className: localClasses.errorSpan,
          type: 'line',
        });
        annotations.push({
          row: aceSpan.startRow,
          column: aceSpan.startCol,
          text: error.text,
          type: 'error',
        });
      });
      break;
  }

  return (
    <div className={clsx(className, localClasses.codeEditor)}>
      <AceEditor
        name="gdlk-editor"
        mode="plain_text"
        theme="terminal" // TODO use a better theme
        width="100%"
        height="100%"
        value={sourceCode}
        annotations={annotations}
        markers={markers}
        onChange={setSourceCode}
      />
    </div>
  );
};

export default CodeEditor;
