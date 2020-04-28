import React, { useState } from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramIde_hardwareSpec } from './__generated__/ProgramIde_hardwareSpec.graphql';
import { makeStyles } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import {
  MachineState,
  OutgoingIdeEvent,
  IdeContextType,
  IdeContext,
} from 'state/ide';
import { SocketSend, SocketConnectionStatus } from 'hooks/useWebSocket';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import NotFoundPage from 'components/NotFoundPage';
import IdeControls from './IdeControls';

const useLocalStyles = makeStyles(({ palette, spacing }) => {
  const border = `2px solid ${palette.divider}`;
  return {
    programIde: {
      width: '100%',
      height: '100%',
      display: 'grid',
      gridTemplateRows: 'auto auto 1fr 1fr',
      gridTemplateColumns: 'repeat(4, 1fr)',
      gridTemplateAreas: `
      'rg rg rg io'
      'ct ct ct io'
      'ed ed ed st'
      'ed ed ed st'
      `,
      border,
    },
    registersInfo: {
      gridArea: 'rg',
      borderBottom: border,
      borderRight: border,
    },
    ioInfo: {
      gridArea: 'io',
      borderBottom: border,
    },
    controls: {
      gridArea: 'ct',
      borderBottom: border,
      borderRight: border,
    },
    editor: {
      gridArea: 'ed',
      borderRight: border,
    },

    stackInfo: {
      gridArea: 'st',
      padding: spacing(1),
    },
  };
});

/**
 * A component to edit and run GDLK programs.
 */
const ProgramIde: React.FC<{
  hardwareSpec: ProgramIde_hardwareSpec;
  machineState?: MachineState;
  wsStatus: SocketConnectionStatus;
  wsSend: SocketSend<OutgoingIdeEvent>;
  relay: RelayProp;
}> = ({ hardwareSpec, wsStatus, machineState, wsSend }) => {
  const localClasses = useLocalStyles();
  const [sourceCode, setSourceCode] = useState<string>(
    hardwareSpec.programSpec?.userProgram?.sourceCode ?? ''
  );

  // Either the program spec or the user program doesn't exist - show 404
  if (!hardwareSpec.programSpec || !hardwareSpec.programSpec.userProgram) {
    return <NotFoundPage />;
  }

  const contextValue: IdeContextType = {
    sourceCode,
    setSourceCode,
    machineState,
    wsStatus,
    wsSend,
  };

  return (
    <IdeContext.Provider value={contextValue}>
      <div className={localClasses.programIde}>
        <RegistersInfo className={localClasses.registersInfo} />
        <IoInfo
          className={localClasses.ioInfo}
          programSpec={hardwareSpec.programSpec}
        />
        <StackInfo
          className={localClasses.stackInfo}
          hardwareSpec={hardwareSpec}
        />
        <IdeControls
          className={localClasses.controls}
          programSpec={hardwareSpec.programSpec}
        />
        <CodeEditor className={localClasses.editor} />
      </div>
    </IdeContext.Provider>
  );
};

export default createFragmentContainer(ProgramIde, {
  hardwareSpec: graphql`
    fragment ProgramIde_hardwareSpec on HardwareSpecNode
      @argumentDefinitions(
        programSlug: { type: "String!" }
        fileName: { type: "String!" }
      ) {
      ...StackInfo_hardwareSpec
      programSpec(slug: $programSlug) {
        id
        slug
        ...IoInfo_programSpec
        ...IdeControls_programSpec @arguments(fileName: $fileName)
        userProgram(fileName: $fileName) {
          fileName
          sourceCode
        }
      }
    }
  `,
});
