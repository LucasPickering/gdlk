import React, { useState, useEffect, useContext } from 'react';
import { makeStyles } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import { IdeContextType, IdeContext } from '@root/state/ide';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import IdeControls from './IdeControls';
import ProgramStatus from './ProgramStatus';
import useDebouncedValue from '@root/hooks/useDebouncedValue';
import PromptOnExit from '@root/components/common/PromptOnExit';
import useCompiler from './useCompiler';
import { hardware } from '@root/data/hardware';
import { PuzzleSolution, Puzzle, HardwareSpec } from '@root/util/types';
import { PuzzleSolutionsContext } from '@root/state/user';

const useLocalStyles = makeStyles(({ palette, spacing }) => {
  const border = `2px solid ${palette.divider}`;
  return {
    programIde: {
      textTransform: 'uppercase',
      width: '100%',
      height: '100%',
      display: 'grid',
      gridTemplateRows: 'auto auto 1fr 1fr',
      gridTemplateColumns: 'auto 1fr auto auto',
      gridTemplateAreas: `
      'io rg rg sk'
      'io st ct sk'
      'io ed ed sk'
      'io ed ed sk'
      `,
      border,
    },
    registersInfo: {
      gridArea: 'rg',
      borderBottom: border,
      borderRight: border,
    },
    ioInfo: {
      gridArea: 'io',
      borderRight: border,
    },
    programStatus: {
      gridArea: 'st',
      borderBottom: border,
    },
    controls: {
      gridArea: 'ct',
      borderBottom: border,
      borderRight: border,
    },
    editor: {
      gridArea: 'ed',
      borderRight: border,
    },

    stackInfo: {
      gridArea: 'sk',
      padding: spacing(1),
    },
  };
});

/**
 * A component to edit and run GDLK programs. This should only be rendered
 * when the necessary GraphQL data has been loaded. This also assumes that
 * the program spec and user program are defined. This should be checked in the
 * parent, otherwise an error will be thrown.
 */
const ProgramIde: React.FC<{
  puzzle: Puzzle;
  puzzleSolution: PuzzleSolution;
}> = ({ puzzle, puzzleSolution }) => {
  const localClasses = useLocalStyles();
  const { setSolutionSourceCode } = useContext(PuzzleSolutionsContext);

  // TODO make this selectable via dropdown
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [hardwareSpec, setHardwareSpec] = useState<HardwareSpec>(hardware.k200);
  const [sourceCode, setSourceCode] = useState<string>(
    puzzleSolution.sourceCode
  );

  const { wasmHardwareSpec, wasmProgramSpec, compiledState, compile, execute } =
    useCompiler({ hardwareSpec, puzzle, sourceCode });

  const [stepping, setStepping] = useState<boolean>(false);

  // When the source changes, save it to local storage and recompile
  // Use a debounce to prevent constant recompilation
  const debouncedSourceCode = useDebouncedValue(sourceCode, 250);
  useEffect(
    () => {
      // TODO comment this
      setSolutionSourceCode(
        puzzle.slug,
        puzzleSolution.fileName,
        debouncedSourceCode
      );

      // Only compile if the source isn't empty. This prevents should an unhelpful
      // error when the user first loads in
      if (debouncedSourceCode.trim()) {
        compile(debouncedSourceCode);
      }
    },
    // Excluding setSolutionSourceCode cause it updates on every loop and
    // memoizing it would just create a mutual recursion loop. Shortcuts!
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [
      puzzle.slug,
      puzzleSolution.fileName,
      debouncedSourceCode,
      // setSolutionSourceCode,
      compile,
    ]
  );

  const contextValue: IdeContextType = {
    wasmHardwareSpec,
    wasmProgramSpec,
    sourceCode,
    compiledState,
    setSourceCode,
    stepping,
    setStepping,
    execute,
    reset: () => compile(sourceCode),
  };

  return (
    <IdeContext.Provider value={contextValue}>
      <div className={localClasses.programIde}>
        <RegistersInfo className={localClasses.registersInfo} />
        <IoInfo className={localClasses.ioInfo} />
        <ProgramStatus className={localClasses.programStatus} />
        <IdeControls className={localClasses.controls} />
        {hardwareSpec.numStacks > 0 && (
          <StackInfo className={localClasses.stackInfo} />
        )}
        <CodeEditor className={localClasses.editor} />

        {/* Prompt on exit for unsaved changes */}
        <PromptOnExit
          when={sourceCode !== puzzleSolution.sourceCode}
          message="You have unsaved changes. Are you sure you want to leave?"
        />
      </div>
    </IdeContext.Provider>
  );
};

export default ProgramIde;
