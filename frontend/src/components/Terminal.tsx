import React, { useEffect } from 'react';
import '@wasmer/wasm-terminal/lib/xterm/xterm.css';
import WasmTerminal from '@wasmer/wasm-terminal/lib/optimized/wasm-terminal.esm';
import { makeStyles } from '@material-ui/core';

const useLocalStyles = makeStyles(() => ({
  terminal: {
    width: '100%',
    height: '100%',
    backgroundColor: 'black',
  },
}));

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
  switch (commandName) {
    case 'callback-command':
      return async (options: CommandArgs, wasmFs: WasmFs): Promise<string> => {
        return `Callback Command Working! Options: ${options}, fs: ${wasmFs}`;
      };
    case 'echo':
      return async (options: CommandArgs): Promise<string> => {
        const gdlk = await import('gdlk_wasm');
        return gdlk.greet(options.args.slice(1).join(' '));
      };
    default:
      return async (options: CommandArgs): Promise<string> => {
        return `command ${options.args[0]} not found`;
      };
  }
};

const Terminal: React.FC = () => {
  const localClasses = useLocalStyles();
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

  return <main className={localClasses.terminal} id="wasm-terminal" />;
};

export default Terminal;
