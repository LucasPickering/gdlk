// Can only use these imports as types, the actual import needs to be async
import type { HardwareSpec, compile } from 'gdlk_wasm';

export type CompiledRes = { instructions: Instruction[] } | { errors: Error[] };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Instruction = any;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Error = any;

type CompileRes = 'ERROR: Invalid Hardware Spec' | CompiledRes;
export default class Compiler {
  static gdlk: {
    compile: typeof compile;
    HardwareSpec: typeof HardwareSpec;
  };

  static async init(): Promise<void> {
    Compiler.gdlk = await import('gdlk_wasm');
  }

  static compile(input: string, hwSpec: HardwareSpec): CompileRes {
    return Compiler.gdlk.compile(input, hwSpec);
  }

  static makeHardwareSpec = (
    num_registers: number,
    num_stacks: number,
    max_stack_length: number
  ): HardwareSpec => {
    return new Compiler.gdlk.HardwareSpec(
      num_registers,
      num_stacks,
      max_stack_length
    );
  };
}
