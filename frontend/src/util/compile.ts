// Can only use these imports as types, the actual import needs to be async
// import type is in ts 3.8 but webpack is sad when i use it for some reason
import { HardwareSpec, compile, Span } from 'gdlk_wasm';
// export interface Span {
//   offset: number;
//   length: number;
//   start_line: number;
//   start_col: number;
//   end_line: number;
//   end_col: number;
// }

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
