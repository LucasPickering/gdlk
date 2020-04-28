import { CircularProgress } from '@material-ui/core';
import React, { useCallback, useState } from 'react';
import { QueryRenderer } from 'react-relay';
import graphql from 'babel-plugin-relay/macro';
import environment from 'util/environment';
import { ProgramIdeViewQuery } from './__generated__/ProgramIdeViewQuery.graphql';
import { useParams } from 'react-router-dom';
import ProgramIde from './ProgramIde';
import useWebSocket, { SocketEventConsumer } from 'hooks/useWebSocket';
import { MachineState, IncomingIdeEvent, OutgoingIdeEvent } from 'state/ide';

interface RouteParams {
  hwSlug: string;
  programSlug: string;
  fileName: string;
}

/**
 * A view that allows the user to edit and run GDLK code.
 */
const ProgramIdeView: React.FC = () => {
  const [machineState, setMachineState] = useState<MachineState | undefined>(
    undefined
  );
  const { hwSlug, programSlug, fileName } = useParams<RouteParams>();
  const { status, send } = useWebSocket<IncomingIdeEvent, OutgoingIdeEvent>(
    `/ws/hardware/${hwSlug}/programs/${programSlug}`,
    // We need to memoize the callbacks to prevent hook triggers
    {
      onMessage: useCallback<
        SocketEventConsumer<IncomingIdeEvent, OutgoingIdeEvent>
      >((send, data) => {
        if (data.type === 'machineState') {
          setMachineState(data.content);
        }
        // TODO handle other data cases
      }, []),
    },
    [hwSlug, programSlug] // Create a new socket when either slug changes
  );

  return (
    <QueryRenderer<ProgramIdeViewQuery>
      environment={environment}
      query={graphql`
        query ProgramIdeViewQuery(
          $hwSlug: String!
          $programSlug: String!
          $fileName: String!
        ) {
          hardwareSpec(slug: $hwSlug) {
            ...ProgramIde_hardwareSpec
              @arguments(programSlug: $programSlug, fileName: $fileName)
          }
        }
      `}
      variables={{ hwSlug, programSlug, fileName }}
      render={({ props, error }) => {
        if (error) {
          return <div>error!</div>;
        }

        if (props?.hardwareSpec) {
          return (
            <ProgramIde
              hardwareSpec={props.hardwareSpec}
              machineState={machineState}
              wsStatus={status}
              wsSend={send}
            />
          );
        }

        return <CircularProgress />;
      }}
    />
  );
};

export default ProgramIdeView;
