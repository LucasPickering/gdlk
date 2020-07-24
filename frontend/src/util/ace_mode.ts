import { getAceInstance } from 'react-ace/lib/editorOptions';
import 'ace-builds/src-noconflict/mode-plain_text';

export class GDLKHighlightRules extends getAceInstance().acequire(
  'ace/mode/text_highlight_rules'
).TextHighlightRules {
  constructor() {
    super();
    this.$rules = {
      // available tokens https://github.com/ajaxorg/ace/wiki/Creating-or-Extending-an-Edit-Mode#commonTokens
      start: [
        {
          token: 'comment',
          regex: ';.*$',
        },
        // {
        //   token: 'text',
        //   regex: '\\-?[a-zA-Z_][a-zA-Z0-9_\\-]*',
        // },
        {
          token: 'meta.function',
          regex: '^\\s*(\\w+:)\\s*$',
        },
        {
          token: 'keyword',
          regex: '^\\s*(\\w+)\\s+',
          next: 'args',
        },
      ],
      args: [
        {
          token: 'variable',
          regex: '\\s*(\\w+)\\s*',
        },
        {
          regex: '$',
          token: 'empty',
          next: 'start',
        },
        {
          regex: '',
          token: 'empty',
          next: 'start',
        },
      ],
    };
  }
}

export default class GDLKMode extends getAceInstance().acequire(
  'ace/mode/plain_text'
).Mode {
  constructor() {
    super();
    this.HighlightRules = GDLKHighlightRules;
    this.lineCommentStart = ';';
  }
}
