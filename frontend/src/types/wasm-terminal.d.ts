// declare module '@wasmer/wasm-terminal';
declare module '@wasmer/wasm-terminal/lib/optimized/wasm-terminal.esm' {
    export default class WasmTerminal {
        constructor(args?: Object);
        open(ele: Element | null);
        fit();
        focus();
    }
}
