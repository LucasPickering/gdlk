import React, { useState, useEffect } from 'react';
import { RelayProp, useFragment } from 'react-relay';
import { graphql } from 'react-relay';
import { makeStyles } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import { IdeContextType, IdeContext } from 'state/ide';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import IdeControls from './IdeControls';
import AutoSaveHandler from './AutoSaveHandler';
import ProgramStatus from './ProgramStatus';
import useDebouncedValue from 'hooks/useDebouncedValue';
import { assertIsDefined } from 'util/guards';
import NotFoundPage from 'components/NotFoundPage';
import { StorageHandler } from 'util/storage';
import useStaticValue from 'hooks/useStaticValue';
import PromptOnExit from 'components/common/PromptOnExit';
import useCompiler from './useCompiler';
// import useCompiler from './useCompiler.ts.disable';

const useLocalStyles = makeStyles(({ palette, spacing }) => {
  const border = `2px solid ${palette.divider}`;
  return {
    programIde: {
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
  queryKey: ProgramIde_query$key;
}> = ({ queryKey }) => {
  const localClasses = useLocalStyles();
  const query = useFragment(
    graphql`
      fragment ProgramIde_hardwareSpec on Query
      @argumentDefinitions(
        hardwareSpecSlug: { type: "String!" }
        puzzleSlug: { type: "String!" }
        fileName: { type: "String!" }
      ) {
        hardwareSpec(slug: $hardwareSpecSlug) {
          id
          numRegisters
          numStacks
          maxStackLength
        }
        puzzle(slug: $puzzleSlug) {
          id
          input
          expectedOutput
          puzzleSolution(fileName: $fileName) {
            id
            sourceCode
            lastModified
            ...AutoSaveHandler_puzzleSolution
          }
        }
      }
    `,
    queryKey
  );

  // If any of the queries things don't exist, freak out!!
  const hardwareSpec = query.hardwareSpec;
  assertIsDefined(hardwareSpec);
  const puzzle = query.puzzle;
  assertIsDefined(puzzle);
  const puzzleSolution = puzzle.puzzleSolution;
  assertIsDefined(puzzleSolution);

  // This will be used to read and write source from/to browser local storage
  const sourceStorageHandler = useStaticValue<StorageHandler<string>>(
    () =>
      new StorageHandler(
        [query.hardwareSpec.id, puzzle.id, puzzleSolution.id, 'source'].join(
          ':'
        )
      )
  );
  const [sourceCode, setSourceCode] = useState<string>(() => {
    // Check local storage for saved source
    const storedSource = sourceStorageHandler.get();
    if (
      storedSource &&
      // Only use the local copy if it's newer than the remote one
      storedSource.lastModified > new Date(puzzleSolution.lastModified)
    ) {
      return storedSource.value;
    }
    return puzzleSolution.sourceCode;
  });

  const { wasmHardwareSpec, wasmProgramSpec, compiledState, compile, execute } =
    useCompiler({ hardwareSpec, sourceCode });

  // When the source changes, save it to local storage and recompile
  // Use a debounce to prevent constant recompilation
  const debouncedSourceCode = useDebouncedValue(sourceCode, 250);
  useEffect(() => {
    sourceStorageHandler.set(debouncedSourceCode);
    compile(debouncedSourceCode);
  }, [sourceStorageHandler, debouncedSourceCode, compile]);

  const contextValue: IdeContextType = {
    wasmHardwareSpec,
    wasmProgramSpec,
    sourceCode,
    compiledState,
    setSourceCode,
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
        <StackInfo className={localClasses.stackInfo} />
        <CodeEditor className={localClasses.editor} />

        <AutoSaveHandler puzzleSolution={puzzleSolution} />
        {/* Prompt on exit for unsaved changes */}
        <PromptOnExit
          when={sourceCode !== puzzleSolution.sourceCode}
          message="You have unsaved changes. Are you sure you want to leave?"
        />
      </div>
    </IdeContext.Provider>
  );
};

/**
 * A thin wrapper around ProgramIde that guarantees that the hardware spec,
 * program spec, and user program are defined before rendering the main
 * component. This makes the logic a lot simpler in the other component.
 */
const ProgramIdeWrapper: React.FC<{
  hardwareSpec: ProgramIde_hardwareSpec;
  relay: RelayProp;
}> = ({ hardwareSpec }) => {
  if (hardwareSpec?.puzzle?.puzzleSolution) {
    return <ProgramIde hardwareSpec={hardwareSpec} />;
  }

  return <NotFoundPage />;
};

export default ProgramIdeWrapper;
