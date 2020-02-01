import React, { useEffect } from 'react';
import '@wasmer/wasm-terminal/lib/xterm/xterm.css';
import WasmTerminal from '@wasmer/wasm-terminal/lib/optimized/wasm-terminal.esm';

interface CommandArgs {
  args: [string];
  callback: Function;
  env: {
    LINES: number;
    COLUMNS: number;
  };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type WasmFs = any; // no idea what this is used for yet but its passed in the command callbacks
type CommandFunction = (args: CommandArgs, wasmFs?: WasmFs) => Promise<string>;

const fetchCommandHandler = async (
  commandName: string
): Promise<CommandFunction | void> => {
  if (commandName === 'callback-command') {
    const callbackCommand = async (
      options: CommandArgs,
      wasmFs: WasmFs
    ): Promise<string> => {
      return `Callback Command Working! Options: ${options}, fs: ${wasmFs}`;
    };
    return callbackCommand;
  } else if (commandName === 'echo') {
    const callbackCommand = async (options: CommandArgs): Promise<string> => {
      // first index is the 'echo' ignore that
      return options.args.slice(1).join(' ');
    };
    return callbackCommand;
  }
};

const Terminal: React.FC = () => {
  const wasmTerminal = React.useRef(
    new WasmTerminal({
      fetchCommand: fetchCommandHandler,
    })
  );
  useEffect(() => {
    const containerElement = document.querySelector('#wasm-terminal');
    wasmTerminal.current.open(containerElement);
    wasmTerminal.current.fit();
    wasmTerminal.current.focus();
  }, []);

  return <main id="wasm-terminal" />;
};

export default Terminal;
