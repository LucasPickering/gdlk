import React, { useState, useCallback } from 'react';
import { RelayProp, createFragmentContainer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import { ProgramIde_hardwareSpec } from './__generated__/ProgramIde_hardwareSpec.graphql';
import { makeStyles, Snackbar } from '@material-ui/core';
import CodeEditor from './CodeEditor';
import RegistersInfo from './RegistersInfo';
import {
  MachineState,
  OutgoingIdeEvent,
  IdeContextType,
  IdeContext,
  IncomingIdeEvent,
} from 'state/ide';
import useWebSocket, { SocketEventConsumer } from 'hooks/useWebSocket';
import IoInfo from './IoInfo';
import StackInfo from './StackInfo';
import NotFoundPage from 'components/NotFoundPage';
import IdeControls from './IdeControls';
import ProgramStatus from './ProgramStatus';
import { Alert } from '@material-ui/lab';

const useLocalStyles = makeStyles(({ palette, spacing }) => {
  const border = `2px solid ${palette.divider}`;
  return {
    programIde: {
      minWidth: '100%',
      minHeight: '100%',
      display: 'grid',
      gridTemplateRows: 'auto auto 1fr 1fr',
      gridTemplateColumns: 'auto 1fr auto auto',
      gridTemplateAreas: `
      'io rg rg sk'
      'io st ct sk'
      'io ed ed sk'
      'io ed ed sk'
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
      borderRight: border,
    },
    programStatus: {
      gridArea: 'st',
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
      gridArea: 'sk',
      padding: spacing(1),
    },
  };
});

/**
 * A component to edit and run GDLK programs. This should only be rendered
 * when the necessary GraphQL data has been loaded.
 */
const ProgramIde: React.FC<{
  hardwareSpec: ProgramIde_hardwareSpec;
  relay: RelayProp;
}> = ({ hardwareSpec }) => {
  const localClasses = useLocalStyles();
  const [machineState, setMachineState] = useState<MachineState | undefined>(
    undefined
  );
  const clearMachineState = useCallback(() => {
    setMachineState(undefined);
  }, []);
  const [sourceCode, setSourceCode] = useState<string>(
    hardwareSpec.programSpec?.userProgram?.sourceCode ?? ''
  );
  const setSourceCodeWrapped = useCallback(
    (newSource: string): void => {
      // Any time we change the source code, we want to invalidate the compiled
      // program.
      setMachineState(undefined);
      setSourceCode(newSource);
    },
    [setMachineState, setSourceCode]
  );

  const hwSlug = hardwareSpec.slug;
  const programSlug = hardwareSpec.programSpec?.slug;

  const { status, send } = useWebSocket<IncomingIdeEvent, OutgoingIdeEvent>(
    // Only connect if the program spec is defined
    programSlug && `/ws/hardware/${hwSlug}/programs/${programSlug}`,

    // We need to memoize the callbacks to prevent hook triggers
    {
      onMessage: useCallback<
        SocketEventConsumer<IncomingIdeEvent, OutgoingIdeEvent>
      >((send, data) => {
        switch (data.type) {
          case 'machineState':
            setMachineState(data.content);
            break;
          // TODO handle other data cases
        }
      }, []),
      onError: clearMachineState,
      onClose: clearMachineState,
    },
    [hwSlug, programSlug] // Create a new socket when either slug changes
  );

  // Either the program spec or the user program doesn't exist - show 404
  if (!hardwareSpec.programSpec || !hardwareSpec.programSpec.userProgram) {
    return <NotFoundPage />;
  }

  const contextValue: IdeContextType = {
    machineState,
    sourceCode,
    setSourceCode: setSourceCodeWrapped,
    wsStatus: status,
    wsSend: send,
  };

  return (
    <IdeContext.Provider value={contextValue}>
      <div className={localClasses.programIde}>
        <RegistersInfo className={localClasses.registersInfo} />
        <IoInfo
          className={localClasses.ioInfo}
          programSpec={hardwareSpec.programSpec}
        />
        <ProgramStatus className={localClasses.programStatus} />
        <IdeControls
          className={localClasses.controls}
          programSpec={hardwareSpec.programSpec}
        />
        <StackInfo
          className={localClasses.stackInfo}
          hardwareSpec={hardwareSpec}
        />
        <CodeEditor className={localClasses.editor} />
      </div>
      <Snackbar open={status === 'connecting'} autoHideDuration={null}>
        <Alert severity="info">Connecting to server...</Alert>
      </Snackbar>
      <Snackbar open={status === 'closedError'}>
        <Alert severity="error">Error, disconnected</Alert>
      </Snackbar>
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
      slug
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
