import { getAceInstance } from 'react-ace/lib/editorOptions';
import 'ace-builds/src-noconflict/mode-plain_text';

export class GDLKHighlightRules extends getAceInstance().require(
  'ace/mode/text_highlight_rules'
).TextHighlightRules {
  constructor() {
    super();
    this.$rules = {
      start: [
        {
          token: 'comment',
          regex: ';.*$',
        },
      ],
    };
  }
}

export default class GDLKMode extends getAceInstance().require(
  'ace/mode/plain_text'
).Mode {
  constructor() {
    super();
    this.HighlightRules = GDLKHighlightRules;
    this.lineCommentStart = ';';
  }
}
