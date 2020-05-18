// Can only use these imports as types, the actual import needs to be async
import {
  HardwareSpec,
  SourceElement,
  compile,
  ProgramSpec,
  Machine,
} from 'gdlk_wasm';
import { LangValue } from 'state/ide';
import { isTypedArray } from './guards';

export interface CompileErrors {
  errors: SourceElement[];
}

/**
 * Type guard for the SourceElement type. Checks if the given value fits the
 * criteria of a SourceElement.
 * @param value The value to check
 * @returns true if the value is a SourceElement, false if not
 */
function isSourceElement(value: object): value is SourceElement {
  // TODO make this more robust after
  // https://github.com/Microsoft/TypeScript/issues/21732
  return 'text' in value && 'span' in value;
}

export interface MachineState {
  programCounter: number;
  input: LangValue[];
  output: LangValue[];
  registers: Record<string, LangValue>;
  stacks: Record<string, LangValue[]>;
  cycleCount: number;
  isComplete: boolean;
  isSuccessful: boolean;
  runtimeError: SourceElement | undefined;
}

/**
 * A wrapper around the wasm Machine type, which makes it easier to use from
 * React. This handles all the interaction with wasm types/values.
 */
export class MachineWrapper {
  private machine: Machine;
  private runtimeError: SourceElement | undefined;
  private _state: MachineState;

  constructor(machine: Machine) {
    this.machine = machine;
    this._state = MachineWrapper.getMachineState(machine, undefined);
  }

  /**
   * A helper to convert wasm values into a JS object that can be handed to React.
   * @param machine The current wasm machine
   * @param runtimeError The current time error, if one has been encountered
   * @return A state object fit for React consumption
   */
  private static getMachineState(
    machine: Machine,
    runtimeError: SourceElement | undefined
  ): MachineState {
    return {
      programCounter: machine.programCounter,
      input: Array.from(machine.input),
      output: Array.from(machine.output),
      registers: machine.registers,
      stacks: machine.stacks,
      cycleCount: machine.cycleCount,
      isComplete: machine.isComplete,
      isSuccessful: machine.isSuccessful,
      runtimeError,
    };
  }

  /**
   * Helper to refresh the current state based on the current wasm values.
   */
  private updateState(): void {
    this._state = MachineWrapper.getMachineState(
      this.machine,
      this.runtimeError
    );
  }

  /**
   * Execute the next instruction. This will update the machine state after
   * the step is run to reflect the new state.
   */
  executeNext(): void {
    try {
      this.machine.executeNext();
    } catch (e) {
      // Make sure this is the error type we expect
      if (isSourceElement(e)) {
        this.runtimeError = e;
      } else {
        // Unrecognized error, throw it back up
        throw e;
      }
    }
    this.updateState();
  }

  get state(): MachineState {
    return this._state;
  }
}

export type CompileResult =
  | { type: 'compiled'; instructions: SourceElement[]; machine: MachineWrapper }
  | { type: 'error'; errors: SourceElement[] };

export class CompilerWrapper {
  static gdlk: {
    compile: typeof compile;
  };

  static async init(): Promise<void> {
    CompilerWrapper.gdlk = await import('gdlk_wasm');
  }

  static compile(
    hardwareSpec: HardwareSpec,
    programSpec: ProgramSpec,
    source: string
  ): CompileResult {
    try {
      const result = CompilerWrapper.gdlk.compile(
        hardwareSpec,
        programSpec,
        source
      );

      return {
        type: 'compiled',
        // Do NOT try to change this to a spread!
        instructions: result.instructions,
        machine: new MachineWrapper(result.machine),
      };
    } catch (e) {
      // Check that the error value matches the expected compile error format
      if (isTypedArray(isSourceElement, e)) {
        return { type: 'error', errors: e };
      }
      // Unknown error, blow up!
      throw e;
    }
  }
}
