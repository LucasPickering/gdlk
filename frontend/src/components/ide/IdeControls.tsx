import React, { useState, useContext, useEffect, useRef } from 'react';
import graphql from 'babel-plugin-relay/macro';
import { makeStyles, Snackbar } from '@material-ui/core';
import {
  Pause as IconPause,
  PlayArrow as IconPlayArrow,
  Refresh as IconRefresh,
  NavigateNext as IconNavigateNext,
  Save as IconSave,
} from '@material-ui/icons';
import { Alert, ToggleButton, ToggleButtonGroup } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import { useMutation } from 'relay-hooks';
import { IdeControls_Mutation } from './__generated__/IdeControls_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { IdeControls_userProgram } from './__generated__/IdeControls_userProgram.graphql';
import clsx from 'clsx';
import IconButton from 'components/common/IconButton';

const DEFAULT_STEP_INTERVAL = 1000; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [2, 20];

const saveUserProgramMutation = graphql`
  mutation IdeControls_Mutation($id: ID!, $sourceCode: String!) {
    updateUserProgram(input: { id: $id, sourceCode: $sourceCode }) {
      userProgramEdge {
        node {
          sourceCode
        }
      }
    }
  }
`;

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  controls: {
    display: 'flex',
    justifyContent: 'end',
    alignItems: 'center',

    backgroundColor: palette.background.default,
  },
  buttons: {
    padding: spacing(1),
  },
  speedSelect: {
    padding: spacing(1),
  },
  speedSelectButton: {
    minWidth: 48,
  },
}));

/**
 * A component to edit and run GDLK programs.
 */
const IdeControls: React.FC<{
  className?: string;
  userProgram: IdeControls_userProgram;
  relay: RelayProp;
}> = ({ className, userProgram }) => {
  const localClasses = useLocalStyles();
  const { compiledState, sourceCode, executeNext, reset } = useContext(
    IdeContext
  );
  const [mutate, { loading: saveLoading }] = useMutation<IdeControls_Mutation>(
    saveUserProgramMutation
  );

  const [saveState, setSaveState] = useState<'success' | 'error' | undefined>();
  const [stepping, setStepping] = useState<boolean>(false);
  const [stepSpeed, setStepSpeed] = useState<number>(STEP_SPEED_OPTIONS[0]);
  const intervalIdRef = useRef<number | undefined>();

  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  useEffect(() => {
    window.clearInterval(intervalIdRef.current);

    if (stepping) {
      intervalIdRef.current = window.setInterval(
        executeNext,
        DEFAULT_STEP_INTERVAL / stepSpeed
      );
    }
  }, [executeNext, stepping, stepSpeed, intervalIdRef]);

  // When the program terminates, stop the stepper
  const terminated = Boolean(machineState?.terminated);
  useEffect(() => {
    if (terminated) {
      window.clearInterval(intervalIdRef.current);
      setStepping(false);
    }
  }, [terminated]);

  return (
    <div className={clsx(localClasses.controls, className)}>
      <div className={localClasses.buttons}>
        <IconButton
          title="Save"
          // Disabled if there aren't any unsaved changes
          disabled={userProgram.sourceCode === sourceCode || saveLoading}
          onClick={() => {
            setSaveState(undefined);
            mutate({
              variables: {
                id: userProgram.id,
                sourceCode,
              },
              onCompleted: () => setSaveState('success'),
              onError: () => setSaveState('error'),
            });
          }}
        >
          <IconSave />
        </IconButton>

        <IconButton
          title="Execute Next Instruction"
          disabled={!machineState || machineState.terminated || stepping}
          onClick={executeNext}
        >
          <IconNavigateNext />
        </IconButton>

        <IconButton
          title={stepping ? 'Pause Execution' : 'Execute Program'}
          disabled={!machineState || machineState.terminated}
          onClick={() => setStepping((prev) => !prev)}
        >
          {stepping ? <IconPause /> : <IconPlayArrow />}
        </IconButton>

        <IconButton
          title={'Reset Program'}
          // Disable if the program hasn't started yet
          disabled={!machineState || machineState.cycleCount === 0}
          onClick={() => {
            reset();
            setStepping(false);
          }}
        >
          <IconRefresh />
        </IconButton>
      </div>

      <ToggleButtonGroup
        className={localClasses.speedSelect}
        value={stepSpeed}
        exclusive
        onChange={(e, newStepSpeed) => setStepSpeed(newStepSpeed)}
      >
        {STEP_SPEED_OPTIONS.map((speed, i) => (
          <ToggleButton
            className={localClasses.speedSelectButton}
            key={speed}
            value={speed}
            aria-label={`${speed} times speed`}
          >
            {'>'.repeat(i + 1)}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>

      {/* Save success notification */}
      <Snackbar
        open={saveState === 'success'}
        onClose={() => setSaveState(undefined)}
      >
        <Alert severity="success">Solution saved</Alert>
      </Snackbar>

      {/* Save error notification */}
      <Snackbar
        open={saveState === 'error'}
        onClose={() => setSaveState(undefined)}
      >
        <Alert severity="error">Error saving program</Alert>
      </Snackbar>
    </div>
  );
};

export default createFragmentContainer(IdeControls, {
  userProgram: graphql`
    fragment IdeControls_userProgram on UserProgramNode {
      id
      sourceCode
    }
  `,
});
