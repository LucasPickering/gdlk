import React, {
  useState,
  useEffect,
  useMemo,
  useCallback,
  useRef,
} from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramIde_hardwareSpec } from './__generated__/ProgramIde_hardwareSpec.graphql';
import { makeStyles } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import { IdeContextType, IdeContext, CompiledState } from 'state/ide';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import IdeControls from './IdeControls';
import ProgramStatus from './ProgramStatus';
import { CompileResult, CompilerWrapper } from 'util/compile';
import { HardwareSpec, ProgramSpec } from 'gdlk_wasm';
import useDebouncedValue from 'hooks/useDebouncedValue';
import { assertIsDefined } from 'util/guards';
import NotFoundPage from 'components/NotFoundPage';

const useLocalStyles = makeStyles(({ palette, spacing }) => {
  const border = `2px solid ${palette.divider}`;
  return {
    programIde: {
      minWidth: '100%',
      minHeight: '100%',
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

  // These values come from wasm. They're read only, so they're safe to share
  // with the whole component tree. They are pointed and there updates won't
  // trigger re-renders, but these values shouldn't be changing while this
  // component tree is mounted.
  const wasmHardwareSpec = useMemo(
    () =>
      new HardwareSpec(
        hardwareSpec.numRegisters,
        hardwareSpec.numStacks,
        hardwareSpec.maxStackLength
      ),
    [hardwareSpec]
  );
  const wasmProgramSpec = useMemo(
    () =>
      new ProgramSpec(
        Int16Array.from(programSpec.input),
        Int16Array.from(programSpec.expectedOutput)
      ),
    [programSpec]
  );

  // This wasm value is NOT safe to share outside this component. It's stored
  // in a ref because it contains pointers, which often don't reflect changed
  // values to React. As such, making it a state field would be useless. This
  // valu ehas to be manually transformed into compiledState after changes.
  const compileResult = useRef<CompileResult | undefined>();

  const [sourceCode, setSourceCode] = useState<string>(userProgram.sourceCode);
  // This is the safe version of the wasm values, which CAN be shared
  const [compiledState, setCompiledState] = useState<
    CompiledState | undefined
  >();

  /**
   * A manual function called to refresh the shared state that's derived from
   * the wasm compiled state. This needs to be called explicitly because there
   * are values in wasm hidden behind pointers. When the values change, the
   * pointers stay the same, so React doesn't recognize that a change has
   * occurred. By copying this state into our own JS objects, we allow React to
   * do its normal updates whenever there's changes.
   *
   * @param newCompileResult The new value of compilation output. If not given
   */
  const updateCompiledState = useCallback(
    (newCompileResult: CompileResult | undefined): void => {
      // Update refs
      compileResult.current = newCompileResult;

      switch (newCompileResult?.type) {
        case 'compiled':
          setCompiledState({
            type: 'compiled',
            instructions: newCompileResult.instructions,
            machineState: newCompileResult.machine.state,
          });
          break;
        case 'error':
          setCompiledState({
            type: 'error',
            errors: newCompileResult.errors,
          });
          break;
        case undefined:
          setCompiledState(undefined);
          break;
      }
    },
    [compileResult]
  );

  /**
   * Compile the given source code, and update the compiled state with the
   * new compiled program.
   *
   * @param source The source code to compile
   */
  const compile = useCallback(
    (source: string): void => {
      updateCompiledState(
        CompilerWrapper.compile(wasmHardwareSpec, wasmProgramSpec, source)
      );
    },
    [wasmHardwareSpec, wasmProgramSpec, updateCompiledState]
  );

  const executeNext = (): void => {
    if (compileResult.current?.type === 'compiled') {
      compileResult.current.machine.executeNext();
      // We need to manually refresh since the wasm pointers won't change
      updateCompiledState(compileResult.current);
    } else {
      // This indicates an FE bug, where we tried to step when not allowed
      throw new Error(
        'Program is not compiled, cannot execute next instruction.'
      );
    }
  };

  // When either spec or the source changes, invalidate the compiled program
  useEffect(() => {
    updateCompiledState(undefined);
  }, [wasmHardwareSpec, wasmProgramSpec, sourceCode, updateCompiledState]);

  // When the source changes, recompile the code
  // Use a debounce to prevent constant recompilation
  const debouncedSourceCode = useDebouncedValue(sourceCode, 250);
  useEffect(() => compile(debouncedSourceCode), [debouncedSourceCode, compile]);

  const contextValue: IdeContextType = {
    wasmHardwareSpec,
    wasmProgramSpec,
    sourceCode,
    compiledState,
    setSourceCode,
    executeNext,
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
          programSpec={programSpec}
        />
        <StackInfo className={localClasses.stackInfo} />
        <CodeEditor className={localClasses.editor} />
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
      numRegisters
      numStacks
      maxStackLength
      programSpec(slug: $programSlug) {
        id
        input
        expectedOutput
        ...IdeControls_programSpec @arguments(fileName: $fileName)
        userProgram(fileName: $fileName) {
          sourceCode
        }
      }
    }
  `,
});
