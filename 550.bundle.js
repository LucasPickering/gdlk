(self.webpackChunkfrontend=self.webpackChunkfrontend||[]).push([[550],{6615:(e,t,r)=>{"use strict";r.d(t,{Z:()=>l});var a=r(9526),n=r(5092);const s=({when:e,message:t})=>((0,a.useEffect)((()=>{if(e){const e=e=>{e.preventDefault(),e.returnValue=""};return window.addEventListener("beforeunload",e),()=>{window.removeEventListener("beforeunload",e)}}}),[e]),a.createElement(n.NL,{when:e,message:t}));s.defaultProps={when:!0};const l=s},5555:(e,t,r)=>{"use strict";r.d(t,{Z:()=>m});var a=r(9526),n=r(2772),s=r(2248),l=r(5048),c=r(3060),o=r(3569);const i=(0,n.Z)((({palette:e,spacing:t})=>({bufferDisplay:{display:"flex",flexDirection:"column","&:not(:first-child)":{paddingLeft:t(1)}},buffer:{display:"flex"},bufferCells:{display:"flex",flexDirection:"column",padding:t(.5),border:`2px solid ${e.divider}`,overflowY:"auto","& + &":{borderLeft:0}},bufferCellsInverted:{flexDirection:"column-reverse"}}))),u=({values:e,maxLength:t,invert:r})=>{const n=i();return a.createElement("div",{className:(0,c.Z)(n.bufferCells,{[n.bufferCellsInverted]:r})},(0,l.Z)(t).map((t=>a.createElement(o.Z,{key:t,value:e[t]}))))},d=({className:e,label:t,values:r,secondaryValues:n,maxLength:l,invert:o})=>{const d=i();return a.createElement("div",{className:(0,c.Z)(d.bufferDisplay,e)},a.createElement(s.Z,{variant:"body2"},t),a.createElement("div",{className:d.buffer},a.createElement(u,{values:r,maxLength:l,invert:o}),n&&a.createElement(u,{values:n,maxLength:l,invert:o})))};d.defaultProps={invert:!1};const m=d},8676:(e,t,r)=>{"use strict";r.d(t,{Z:()=>m});var a=r(9526),n=r(2772),s=r(3060),l=r(4180),c=r(2262),o=(r(4803),r(7103));r(2566);class i extends(0,o.getAceInstance)().require("ace/mode/text_highlight_rules").TextHighlightRules{constructor(){super(),this.$rules={start:[{token:"comment",regex:";.*$"}]}}}class u extends(0,o.getAceInstance)().require("ace/mode/plain_text").Mode{constructor(){super(),this.HighlightRules=i,this.lineCommentStart=";"}}const d=(0,n.Z)((({palette:e})=>({codeEditor:{display:"flex",overflowY:"auto",backgroundColor:e.background.default,textTransform:"none"},errorSpan:{position:"absolute",backgroundColor:e.error.dark},activeInstruction:{position:"absolute",backgroundColor:e.grey[600]}}))),m=({className:e})=>{const t=d(),{compiledState:r,sourceCode:n,setSourceCode:o}=(0,a.useContext)(l.R),i=[],m=[];switch(null==r?void 0:r.type){case"compiled":{const{machineState:{programCounter:e,runtimeError:a}}=r,n=r.instructions[e];if(n&&i.push(Object.assign(Object.assign({},(0,l.k)(n.span)),{className:t.activeInstruction,type:"fullLine"})),a){const e=(0,l.k)(a.span);i.push(Object.assign(Object.assign({},e),{className:t.activeInstruction,type:"fullLine"})),m.push({row:e.startRow,column:e.startCol,text:a.text,type:"error"})}}break;case"error":r.errors.forEach((e=>{const r=(0,l.k)(e.span);i.push(Object.assign(Object.assign({},r),{className:t.errorSpan,type:"text"})),m.push({row:r.startRow,column:r.startCol,text:e.text,type:"error"})}))}return a.createElement("div",{className:(0,s.Z)(e,t.codeEditor)},a.createElement(c.ZP,{name:"gdlk-editor",mode:"text",theme:"terminal",width:"100%",height:"100%",onLoad:e=>{const t=new u;e.getSession().setMode(t)},value:n,annotations:m,markers:i,onChange:o}))}},1717:(e,t,r)=>{"use strict";r.d(t,{Z:()=>S});var a=r(9526),n=r(2772),s=r(5349),l=r(1858),c=r(5433),o=r(341),i=r(9322),u=r(1612),d=r(7725),m=r(4180),p=r(3060),g=r(9110),f=r(3167),v=r(4723);const h=(0,n.Z)({loading:{position:"absolute",top:"50%",left:"50%",marginTop:-12,marginLeft:-12}}),b=e=>{var{title:t,loading:r,color:n,children:s}=e,l=function(e,t){var r={};for(var a in e)Object.prototype.hasOwnProperty.call(e,a)&&t.indexOf(a)<0&&(r[a]=e[a]);if(null!=e&&"function"==typeof Object.getOwnPropertySymbols){var n=0;for(a=Object.getOwnPropertySymbols(e);n<a.length;n++)t.indexOf(a[n])<0&&Object.prototype.propertyIsEnumerable.call(e,a[n])&&(r[a[n]]=e[a[n]])}return r}(e,["title","loading","color","children"]);const c=h(),o=a.createElement(g.Z,Object.assign({"aria-label":t,color:n},r?{"aria-busy":"true","aria-live":"polite"}:{},l),r?a.createElement(f.Z,{className:c.loading,color:"default"===n?"inherit":n,size:24}):s);return t?a.createElement(v.ZP,{title:t},a.createElement("span",null,o)):o};b.defaultProps={loading:!1};const E=b,x=[2,20],y=(0,n.Z)((({palette:e,spacing:t})=>({controls:{display:"flex",justifyContent:"end",alignItems:"center",backgroundColor:e.background.default},buttons:{padding:t(1)},speedSelect:{padding:t(1)},speedSelectButton:{minWidth:48}}))),S=({className:e})=>{const t=y(),{compiledState:r,stepping:n,setStepping:g,execute:f,reset:v}=(0,a.useContext)(m.R),h=(0,a.useCallback)((()=>f(!1)),[f]),[b,S]=(0,a.useState)(x[0]),Z=(0,a.useRef)(),w="compiled"===(null==r?void 0:r.type)?r.machineState:void 0;(0,a.useEffect)((()=>{window.clearInterval(Z.current),n&&(Z.current=window.setInterval(h,1e3/b))}),[h,n,b,Z]);const k=Boolean(null==w?void 0:w.terminated);return(0,a.useEffect)((()=>{k&&(window.clearInterval(Z.current),g(!1))}),[k,g]),a.createElement("div",{className:(0,p.Z)(t.controls,e)},a.createElement("div",{className:t.buttons},a.createElement(E,{title:"Execute Next Instruction",disabled:!w||w.terminated||n,onClick:h},a.createElement(s.Z,null)),a.createElement(E,{title:n?"Pause Execution":"Execute Program",disabled:!w||w.terminated,onClick:()=>g((e=>!e))},n?a.createElement(l.Z,null):a.createElement(c.Z,null)),a.createElement(E,{title:"Execute to End",disabled:!w||w.terminated||n,onClick:()=>f(!0)},a.createElement(o.Z,null)),a.createElement(E,{title:"Reset Program",disabled:!w||0===w.cycleCount,onClick:()=>{v(),g(!1)}},a.createElement(i.Z,null))),a.createElement(u.Z,{className:t.speedSelect,value:b,exclusive:!0,onChange:(e,t)=>{null!==t&&S(t)}},x.map(((e,r)=>a.createElement(d.Z,{className:t.speedSelectButton,key:e,value:e,"aria-label":`${e} times speed`},">".repeat(r+1))))))}},9433:(e,t,r)=>{"use strict";r.d(t,{Z:()=>i});var a=r(9526),n=r(4180),s=r(2772),l=r(5555),c=r(3060);const o=(0,s.Z)((({palette:e,spacing:t})=>({ioBuffers:{backgroundColor:e.background.default,padding:t(1),display:"flex"}}))),i=({className:e})=>{var t,r;const s=o(),{wasmProgramSpec:i,compiledState:u}=(0,a.useContext)(n.R),d=Array.from(i.input),m=Array.from(i.expectedOutput),p="compiled"===(null==u?void 0:u.type)?u.machineState:void 0;return a.createElement("div",{className:(0,c.Z)(s.ioBuffers,e)},a.createElement(l.Z,{label:"Input",values:null!==(t=null==p?void 0:p.input)&&void 0!==t?t:d,maxLength:d.length}),a.createElement(l.Z,{label:"Output",values:m,secondaryValues:null!==(r=null==p?void 0:p.output)&&void 0!==r?r:[],maxLength:m.length}))}},3569:(e,t,r)=>{"use strict";r.d(t,{Z:()=>o});var a=r(9526),n=r(2772),s=r(2248),l=r(3060);const c=(0,n.Z)((()=>({langValueDisplay:{minWidth:60,textAlign:"right",lineHeight:1.1}}))),o=({className:e,value:t})=>{const r=c();return a.createElement(s.Z,{className:(0,l.Z)(e,r.langValueDisplay)},null!=t?t:"-")}},7657:(e,t,r)=>{"use strict";r.a(e,(async e=>{r.d(t,{Z:()=>E});var a=r(9526),n=r(2772),s=r(8676),l=r(8077),c=r(4180),o=r(9433),i=r(9609),u=r(1717),d=r(6368),m=r(8157),p=r(6615),g=r(9990),f=r(5965),v=r(3505),h=e([d,g]);[d,g]=h.then?await h:h;const b=(0,n.Z)((({palette:e,spacing:t})=>{const r=`2px solid ${e.divider}`;return{programIde:{textTransform:"uppercase",width:"100%",height:"100%",display:"grid",gridTemplateRows:"auto auto 1fr 1fr",gridTemplateColumns:"auto 1fr auto auto",gridTemplateAreas:"\n      'io rg rg sk'\n      'io st ct sk'\n      'io ed ed sk'\n      'io ed ed sk'\n      ",border:r},registersInfo:{gridArea:"rg",borderBottom:r,borderRight:r},ioInfo:{gridArea:"io",borderRight:r},programStatus:{gridArea:"st",borderBottom:r},controls:{gridArea:"ct",borderBottom:r,borderRight:r},editor:{gridArea:"ed",borderRight:r},stackInfo:{gridArea:"sk",padding:t(1)}}})),E=({puzzle:e})=>{var t;const r=b(),[n,h]=(0,f.FV)((0,v.FZ)({puzzleSlug:e.slug})),E=(0,f.sJ)(v.B4),[x,y]=(0,a.useState)(n.sourceCode),{wasmHardwareSpec:S,wasmProgramSpec:Z,compiledState:w,compile:k,execute:C}=(0,g.Z)({hardwareSpec:E,puzzle:e,sourceCode:x}),[N,R]=(0,a.useState)(!1),I=(0,m.Z)(x,250);(0,a.useEffect)((()=>{h((e=>Object.assign(Object.assign({},e),{sourceCode:I}))),I.trim()&&k(I)}),[I,k,h]);const O="compiled"===(null==w?void 0:w.type)?w.machineState:void 0,P=null!==(t=null==O?void 0:O.successful)&&void 0!==t&&t;(0,a.useEffect)((()=>{P&&h((e=>Object.assign(Object.assign({},e),{solved:!0})))}),[P,h]);const z={wasmHardwareSpec:S,wasmProgramSpec:Z,sourceCode:x,compiledState:w,setSourceCode:y,stepping:N,setStepping:R,execute:C,reset:()=>k(x)};return a.createElement(c.R.Provider,{value:z},a.createElement("div",{className:r.programIde},a.createElement(l.Z,{className:r.registersInfo}),a.createElement(o.Z,{className:r.ioInfo}),a.createElement(d.Z,{className:r.programStatus}),a.createElement(u.Z,{className:r.controls}),E.numStacks>0&&a.createElement(i.Z,{className:r.stackInfo}),a.createElement(s.Z,{className:r.editor}),a.createElement(p.Z,{when:x!==n.sourceCode,message:"You have unsaved changes. Are you sure you want to leave?"})))}}))},4550:(e,t,r)=>{"use strict";r.a(e,(async e=>{r.r(t),r.d(t,{default:()=>u});var a=r(9526),n=r(5092),s=r(7657),l=r(7904),c=r(1091),o=r(250),i=e([s]);s=(i.then?await i:i)[0];const u=()=>{const{puzzleSlug:e}=(0,n.UO)(),t=o.$[e];return a.createElement(c.Z,{fullScreen:!0,navProps:{backLink:{to:`/puzzles/${e}`,label:"Back to Puzzle"}}},t?a.createElement(s.Z,{puzzle:t}):a.createElement(l.Z,null))}}))},6368:(e,t,r)=>{"use strict";r.a(e,(async e=>{r.d(t,{Z:()=>i});var a=r(9526),n=r(4180),s=r(3060),l=r(2772);const{FailureReason:c}=await r.e(356).then(r.bind(r,5356)),o=(0,l.Z)((({spacing:e})=>({programStatus:{padding:e(1)}}))),i=({className:e})=>{var t;const r=o(),{stepping:l,compiledState:i}=(0,a.useContext)(n.R),u="compiled"===(null==i?void 0:i.type)?i.machineState:void 0;return a.createElement("div",{className:(0,s.Z)(e,r.programStatus)},a.createElement("div",null,"CPU Cycles: ",null!==(t=null==u?void 0:u.cycleCount)&&void 0!==t?t:"–"),a.createElement("div",null,function(e,t){var r;if(!e)return"";if("error"===e.type)return"Error - Compilation failure";const a=e.machineState;if(!a.terminated&&!t)return"Ready";if(!a.terminated&&t)return"Running...";if(a.successful)return"Success";switch(a.failureReason){case c.RuntimeError:return`Error - ${null===(r=a.runtimeError)||void 0===r?void 0:r.text}`;case c.RemainingInput:return"Failure - Values remain in input buffer";case c.IncorrectOutput:return"Failure - Output did not match expectation";default:return console.error(`Unexpected program failure state: ${a.failureReason}`),"Failure"}}(i,l)))};e()}),1)},8077:(e,t,r)=>{"use strict";r.d(t,{Z:()=>d});var a=r(9526),n=r(4180),s=r(2772),l=r(2248),c=r(3569),o=r(3060);const i=(0,s.Z)((({palette:e,spacing:t})=>({registers:{padding:t(1),backgroundColor:e.background.default,display:"flex"},register:{display:"flex",flexDirection:"column",alignItems:"flex-end","&:not(:first-child)":{paddingLeft:t(2)}}}))),u=({name:e,value:t})=>{const r=i();return a.createElement("div",{className:r.register},a.createElement(l.Z,null,e),a.createElement(c.Z,{value:t}))},d=({className:e})=>{const t=i(),{wasmHardwareSpec:r,compiledState:s}=(0,a.useContext)(n.R),l="compiled"===(null==s?void 0:s.type)?s.machineState:void 0;return a.createElement("div",{className:(0,o.Z)(t.registers,e)},r.registers.map((e=>a.createElement(u,{key:e,name:e,value:null==l?void 0:l.registers[e]}))))}},9609:(e,t,r)=>{"use strict";r.d(t,{Z:()=>i});var a=r(9526),n=r(4180),s=r(2772),l=r(5555),c=r(3060);const o=(0,s.Z)((({palette:e,spacing:t})=>({stackInfo:{display:"flex",flexDirection:"column",padding:t(1),backgroundColor:e.background.default,height:"100%"},stack:{maxHeight:"100%",paddingLeft:"0 !important"}}))),i=({className:e})=>{const t=o(),{wasmHardwareSpec:r,compiledState:s}=(0,a.useContext)(n.R),i="compiled"===(null==s?void 0:s.type)?s.machineState:void 0;return a.createElement("div",{className:(0,c.Z)(t.stackInfo,e)},r.stacks.map((e=>{var n;return a.createElement(l.Z,{className:t.stack,key:e,invert:!0,label:e,values:null!==(n=null==i?void 0:i.stacks[e])&&void 0!==n?n:[],maxLength:r.max_stack_length})})))}},9990:(e,t,r)=>{"use strict";r.a(e,(async e=>{r.d(t,{Z:()=>i});var a=r(5097),n=r(9526),s=r(2191),l=e([s]);s=(l.then?await l:l)[0];const{HardwareSpec:c,ProgramSpec:o}=await r.e(356).then(r.bind(r,5356)),i=({hardwareSpec:e,puzzle:t,sourceCode:r})=>{const l=(0,a.Z)((()=>new c(e.numRegisters,e.numStacks,e.maxStackLength))),i=(0,a.Z)((()=>new o(Int32Array.from(t.input),Int32Array.from(t.expectedOutput)))),u=(0,n.useRef)(),[d,m]=(0,n.useState)(),p=(0,n.useCallback)((e=>{switch(u.current=e,null==e?void 0:e.type){case"compiled":m({type:"compiled",instructions:e.instructions,machineState:e.machine.state});break;case"error":m({type:"error",errors:e.errors});break;case void 0:m(void 0)}}),[u]),g=(0,n.useCallback)((e=>{p(s.S.compile(l,i,e))}),[l,i,p]),f=(0,n.useCallback)(((e=!1)=>{var t;if("compiled"!==(null===(t=u.current)||void 0===t?void 0:t.type))throw new Error("Program is not compiled, cannot execute next instruction.");u.current.machine.execute(e),p(u.current)}),[u,p]);return(0,n.useEffect)((()=>()=>p(void 0)),[l,i,r,p]),{wasmHardwareSpec:l,wasmProgramSpec:i,compiledState:d,compile:g,execute:f}};e()}),1)},8157:(e,t,r)=>{"use strict";r.d(t,{Z:()=>s});var a=r(9526),n=r(1671);const s=(e,t)=>{const[r,s]=(0,a.useState)(e),l=(0,a.useCallback)((0,n.Z)((e=>{s(e)}),t),[t]);return(0,a.useEffect)((()=>{l(e)}),[e,l]),r}},5097:(e,t,r)=>{"use strict";r.d(t,{Z:()=>n});var a=r(9526);const n=e=>{const t=(0,a.useRef)({initialized:!1,value:void 0});return t.current.initialized||(t.current={initialized:!0,value:e()}),t.current.value}},4180:(e,t,r)=>{"use strict";function a(e){return{startRow:e.start_line-1,startCol:e.start_col-1,endRow:e.end_line-1,endCol:e.end_col}}r.d(t,{k:()=>a,R:()=>n});const n=r(9526).createContext({})},2191:(e,t,r)=>{"use strict";r.a(e,(async e=>{r.d(t,{S:()=>c});var a=r(1003);const n=await r.e(356).then(r.bind(r,5356));function s(e){return"text"in e&&"span"in e}class l{constructor(e){this.machine=e,this._state=l.getMachineState(e)}static getMachineState(e){return{programCounter:e.programCounter,input:Array.from(e.input),output:Array.from(e.output),registers:e.registers,stacks:e.stacks,cycleCount:e.cycleCount,terminated:e.terminated,successful:e.successful,runtimeError:e.error,failureReason:e.failureReason}}updateState(){this._state=l.getMachineState(this.machine)}execute(e){e?this.machine.executeAll():this.machine.executeNext(),this.updateState()}get state(){return this._state}}class c{static compile(e,t,r){try{const a=n.compile(e,t,r);return{type:"compiled",instructions:a.instructions,machine:new l(a.machine)}}catch(e){if((0,a.f)(s,e))return{type:"error",errors:e};throw e}}}e()}),1)},1003:(e,t,r)=>{"use strict";function a(e,t){return t instanceof Array&&t.every((t=>e(t)))}r.d(t,{f:()=>a})}}]);