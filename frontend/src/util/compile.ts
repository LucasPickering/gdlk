// Can only use these imports as types, the actual import needs to be async
import type {
  HardwareSpec,
  SourceElement,
  ProgramSpec,
  Machine,
} from "gdlk_wasm";
import { LangValue } from "@root/state/ide";
import { isTypedArray } from "./guards";
const gdlk = await import("gdlk_wasm");

export interface CompileErrors {
  errors: SourceElement[];
}

/**
 * Type guard for the SourceElement type. Checks if the given value fits the
 * criteria of a SourceElement.
 * @param value The value to check
 * @returns true if the value is a SourceElement, false if not
 */
// eslint-disable-next-line @typescript-eslint/ban-types
function isSourceElement(value: object): value is SourceElement {
  // TODO make this more robust after
  // https://github.com/Microsoft/TypeScript/issues/21732
  return "text" in value && "span" in value;
}

export interface MachineState {
  programCounter: number;
  input: LangValue[];
  output: LangValue[];
  registers: Record<string, LangValue>;
  stacks: Record<string, LangValue[]>;
  cycleCount: number;
  terminated: boolean;
  successful: boolean;
  runtimeError: SourceElement | undefined;
  failureReason: number | undefined;
}

/**
 * A wrapper around the wasm Machine type, which makes it easier to use from
 * React. This handles all the interaction with wasm types/values.
 */
export class MachineWrapper {
  private machine: Machine;
  private _state: MachineState;

  constructor(machine: Machine) {
    this.machine = machine;
    this._state = MachineWrapper.getMachineState(machine);
  }

  /**
   * A helper to convert wasm values into a JS object that can be handed to React.
   * @param machine The current wasm machine
   * @param runtimeError The current time error, if one has been encountered
   * @return A state object fit for React consumption
   */
  private static getMachineState(machine: Machine): MachineState {
    return {
      programCounter: machine.programCounter,
      input: Array.from(machine.input),
      output: Array.from(machine.output),
      registers: machine.registers,
      stacks: machine.stacks,
      cycleCount: machine.cycleCount,
      terminated: machine.terminated,
      successful: machine.successful,
      runtimeError: machine.error,
      failureReason: machine.failureReason,
    };
  }

  /**
   * Helper to refresh the current state based on the current wasm values.
   */
  private updateState(): void {
    this._state = MachineWrapper.getMachineState(this.machine);
  }

  /**
   * Execute the next instruction, or if specified, all remaining instructions.
   * This will update the machine state after the step is run to reflect the new
   * state.
   */
  execute(executeAll: boolean): void {
    if (executeAll) {
      this.machine.executeAll();
    } else {
      this.machine.executeNext();
    }
    this.updateState();
  }

  get state(): MachineState {
    return this._state;
  }
}

export type CompileResult =
  | { type: "compiled"; instructions: SourceElement[]; machine: MachineWrapper }
  | { type: "error"; errors: SourceElement[] };

export class Compiler {
  static compile(
    hardwareSpec: HardwareSpec,
    programSpec: ProgramSpec,
    source: string
  ): CompileResult {
    try {
      const result = gdlk.compile(hardwareSpec, programSpec, source);

      return {
        type: "compiled",
        instructions: result.instructions,
        machine: new MachineWrapper(result.machine),
      };
    } catch (e) {
      // Check that the error value matches the expected compile error format
      if (isTypedArray(isSourceElement, e)) {
        return { type: "error", errors: e };
      }
      // Unknown error, blow up!
      throw e;
    }
  }
}
