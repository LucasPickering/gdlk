import React from "react";
import '../../node_modules/@wasmer/wasm-terminal/lib/xterm/xterm.css'
import WasmTerminal from "@wasmer/wasm-terminal/lib/optimized/wasm-terminal.esm";

interface callbackCommandArgs {
  args: [String];
  callback: Function,
  env: {
    LINES: number,
    COLUMNS: number
  }
}

const fetchCommandHandler = async (commandName: String) => {
  if (commandName === 'callback-command') {
    const callbackCommand = async (options: callbackCommandArgs, wasmFs: any) => {
      return `Callback Command Working! Options: ${options}, fs: ${wasmFs}`;
    };
    return callbackCommand;
  } else if (commandName === 'echo') {
    const callbackCommand = async (options: callbackCommandArgs, wasmFs: any) => {
      // first index is the 'echo' ignore that
      return options.args.slice(1).join(' ');
    };
    return callbackCommand;
  };
};
export default class Terminal extends React.Component {
  wasmTerminal: WasmTerminal;
  constructor(props: any) {
    super(props);
    this.wasmTerminal = new WasmTerminal({
      fetchCommand: fetchCommandHandler
    });
  }

  componentDidMount() {
    const containerElement = document.querySelector("#wasm-terminal");
    this.wasmTerminal.open(containerElement);
    this.wasmTerminal.fit();
    this.wasmTerminal.focus();
  }

  render() {
    return <main id="wasm-terminal"></main>
  }
}

