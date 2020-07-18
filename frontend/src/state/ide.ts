import React from 'react';
import { Span, HardwareSpec, ProgramSpec, SourceElement } from 'gdlk_wasm';
import { MachineState } from 'util/compile';

export type LangValue = number;

/**
 * A span in the format that Ace likes. This is distinctly different from the
 * Span we get from gdlk, and we'll need to explicitly convert between the two.
 */
export interface AceSpan {
  startRow: number;
  startCol: number;
  endRow: number;
  endCol: number;
}

export function gdlkSpanToAce(span: Span): AceSpan {
  return {
    startRow: span.start_line - 1,
    startCol: span.start_col - 1,
    endRow: span.end_line - 1,
    endCol: span.end_col,
  };
}

export type CompiledState =
  | {
      type: 'compiled';
      instructions: SourceElement[];
      machineState: MachineState;
    }
  | { type: 'error'; errors: SourceElement[] };

/**
 * The context data that gets shared throughout the IDE. This is designed in
 * order to isolate the wasm logic from all  children. Wasm interactions can get
 * weird, so we want to manage that all in the root. All accesses by the
 * children should be safe.
 */
export interface IdeContextType {
  wasmHardwareSpec: HardwareSpec;
  wasmProgramSpec: ProgramSpec;
  sourceCode: string;
  compiledState: CompiledState | undefined;
  setSourceCode: (newSourceCode: string) => void;
  execute: (executeAll?: boolean) => void;
  reset: () => void;
}

export const IdeContext = React.createContext<IdeContextType>(
  {} as IdeContextType // this default value never gets used so this is "safe"
);
