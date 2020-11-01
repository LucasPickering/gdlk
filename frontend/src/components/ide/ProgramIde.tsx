import React, { useState, useEffect } from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramIde_hardwareSpec } from './__generated__/ProgramIde_hardwareSpec.graphql';
import { makeStyles } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import { IdeContextType, IdeContext } from 'state/ide';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import IdeControls from './IdeControls';
import ProgramStatus from './ProgramStatus';
import useDebouncedValue from 'hooks/useDebouncedValue';
import { assertIsDefined } from 'util/guards';
import NotFoundPage from 'components/NotFoundPage';
import { StorageHandler } from 'util/storage';
import useStaticValue from 'hooks/useStaticValue';
import PromptOnExit from 'components/common/PromptOnExit';
import useCompiler from './useCompiler';

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
  hardwareSpec: ProgramIde_hardwareSpec;
}> = ({ hardwareSpec }) => {
  const localClasses = useLocalStyles();

  // If the program spec or the user program doesn't exist, freak out!!
  const programSpec = hardwareSpec.programSpec;
  assertIsDefined(programSpec);
  const userProgram = programSpec.userProgram;
  assertIsDefined(userProgram);

  // This will be used to read and write source from/to browser local storage
  const sourceStorageHandler = useStaticValue<StorageHandler<string>>(
    () =>
      new StorageHandler(
        [hardwareSpec.id, programSpec.id, userProgram.id, 'source'].join(':')
      )
  );
  const [sourceCode, setSourceCode] = useState<string>(() => {
    // Check local storage for saved source
    const storedSource = sourceStorageHandler.get();
    if (
      storedSource &&
      // Only use the local copy if it's newer than the remote one
      storedSource.lastModified > new Date(userProgram.lastModified)
    ) {
      return storedSource.value;
    }
    return userProgram.sourceCode;
  });

  const {
    wasmHardwareSpec,
    wasmProgramSpec,
    compiledState,
    compile,
    execute,
  } = useCompiler({ hardwareSpec, sourceCode });

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
        <IdeControls
          className={localClasses.controls}
          userProgram={userProgram}
        />
        <StackInfo className={localClasses.stackInfo} />
        <CodeEditor className={localClasses.editor} />

        {/* Prompt on exit for unsaved changes */}
        <PromptOnExit
          when={sourceCode !== userProgram.sourceCode}
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
  if (hardwareSpec?.programSpec?.userProgram) {
    return <ProgramIde hardwareSpec={hardwareSpec} />;
  }

  return <NotFoundPage />;
};

export default createFragmentContainer(ProgramIdeWrapper, {
  hardwareSpec: graphql`
    fragment ProgramIde_hardwareSpec on HardwareSpecNode
      @argumentDefinitions(
        programSlug: { type: "String!" }
        fileName: { type: "String!" }
      ) {
      id
      numRegisters
      numStacks
      maxStackLength
      programSpec(slug: $programSlug) {
        id
        input
        expectedOutput
        userProgram(fileName: $fileName) {
          id
          sourceCode
          lastModified
          ...IdeControls_userProgram
        }
      }
    }
  `,
});
