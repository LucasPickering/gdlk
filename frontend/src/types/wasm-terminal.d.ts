// declare module '@wasmer/wasm-terminal';
declare module '@wasmer/wasm-terminal/lib/optimized/wasm-terminal.esm' {
  export default class WasmTerminal {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    constructor(args?: Record<string, any>);
    open(ele: Element | null);
    fit();
    focus();
    scrollToCursor();
    print(msg: string, sync?: boolean);
    runCommand(line: string);
    destroy();
  }
}
