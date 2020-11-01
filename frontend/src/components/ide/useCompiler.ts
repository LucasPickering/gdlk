import { HardwareSpec, ProgramSpec } from 'gdlk_wasm';
import useStaticValue from 'hooks/useStaticValue';
import { useCallback, useEffect, useRef, useState } from 'react';
import { CompiledState } from 'state/ide';
import { CompileResult, CompilerWrapper } from 'util/compile';
import { assertIsDefined } from 'util/guards';
import { ProgramIde_hardwareSpec } from './__generated__/ProgramIde_hardwareSpec.graphql';

interface Input {
  hardwareSpec: ProgramIde_hardwareSpec;
  sourceCode: string;
}

interface Output {
  wasmHardwareSpec: HardwareSpec;
  wasmProgramSpec: ProgramSpec;
  compiledState: CompiledState | undefined;
  compile: (source: string) => void;
  execute: (executeAll?: boolean) => void;
}

/**
 * A helper hook that handles a lot of state management for the compiler.
 * Because a lot of the state is stored in Wasm, we need some extra logic
 * surrounding it to make sure it interacts properly with React. This hook
 * encapsulates all that state, to make it easier to work with externally.
 * @param hardwareSpec Hardware being compiled on
 * @param sourceCode Current source code, as shown in the editor (NOT necessarily
 *  what is saved server-side)
 */
const useCompiler = ({ hardwareSpec, sourceCode }: Input): Output => {
  // If the program spec or the user program doesn't exist, freak out!!
  const programSpec = hardwareSpec.programSpec;
  assertIsDefined(programSpec);
  const userProgram = programSpec.userProgram;
  assertIsDefined(userProgram);

  // These values come from wasm. They're read only, so they're safe to share
  // with the whole component tree. They are pointers and therefore updates
  // won't trigger re-renders, but these values shouldn't be changing while
  // this component tree is mounted anyway.
  const wasmHardwareSpec = useStaticValue(
    () =>
      new HardwareSpec(
        hardwareSpec.numRegisters,
        hardwareSpec.numStacks,
        hardwareSpec.maxStackLength
      )
  );
  const wasmProgramSpec = useStaticValue(
    () =>
      new ProgramSpec(
        Int32Array.from(programSpec.input),
        Int32Array.from(programSpec.expectedOutput)
      )
  );

  // This wasm value is NOT safe to share outside this component. It's stored
  // in a ref because it contains pointers, which often don't reflect changed
  // values to React. As such, making it a state field would be useless. This
  // value has to be manually transformed into compiledState after changes.
  const compileResult = useRef<CompileResult | undefined>();

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

  const execute = useCallback(
    (executeAll: boolean = false): void => {
      if (compileResult.current?.type === 'compiled') {
        compileResult.current.machine.execute(executeAll);
        // We need to manually refresh since the wasm pointers won't change
        updateCompiledState(compileResult.current);
      } else {
        // This indicates an FE bug, where we tried to step when not allowed
        throw new Error(
          'Program is not compiled, cannot execute next instruction.'
        );
      }
    },
    [compileResult, updateCompiledState]
  );

  // When either spec or the source changes, invalidate the compiled program
  useEffect(() => {
    // Do this as a post-effect so that it doesn't run on first render. That
    // prevents us wiping out state right after we compile
    return () => updateCompiledState(undefined);
  }, [wasmHardwareSpec, wasmProgramSpec, sourceCode, updateCompiledState]);

  return { wasmHardwareSpec, wasmProgramSpec, compiledState, compile, execute };
};

export default useCompiler;
