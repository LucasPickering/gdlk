import React from 'react';
import { SocketSend, SocketConnectionStatus } from 'hooks/useWebSocket';

export type LangValue = number;

/**
 * See `Machine` type in the `core` crate for a description of fields.
 */
export interface MachineState {
  programCounter: number;
  input: LangValue[];
  output: LangValue[];
  registers: Record<string, LangValue>;
  stacks: Record<string, LangValue[]>;
  cycleCount: number;
  isComplete: boolean;
  isSuccessful: boolean;
}

export type OutgoingIdeEvent =
  | {
      type: 'compile';
      content: { sourceCode: string };
    }
  | { type: 'step' };

export type IncomingIdeEvent =
  | {
      type: 'machineState';
      content: MachineState;
    }
  | { type: 'malformedMessage'; content: string }
  | {
      type: 'compileError';
      // TODO add content field
    }
  | {
      type: 'runtimeError';
      // TODO add content field
    }
  | { type: 'noCompilation' };

export interface IdeContextType {
  machineState: MachineState | undefined;
  sourceCode: string;
  setSourceCode: (newSourceCode: string) => void;
  wsStatus: SocketConnectionStatus;
  wsSend: SocketSend<OutgoingIdeEvent>;
}

export const IdeContext = React.createContext<IdeContextType>(
  {} as IdeContextType // this default value never gets used so this is "safe"
);
