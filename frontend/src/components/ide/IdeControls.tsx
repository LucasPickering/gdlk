import React, {
  useState,
  useContext,
  useRef,
  useCallback,
  useEffect,
} from 'react';
import graphql from 'babel-plugin-relay/macro';
import {
  makeStyles,
  Button,
  Snackbar,
  FormControlLabel,
  RadioGroup,
  Radio,
} from '@material-ui/core';
import { Alert } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import { useMutation } from 'relay-hooks';
import { IdeControls_Mutation } from './__generated__/IdeControls_Mutation.graphql';
import LoadingButton from 'components/common/LoadingButton';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { IdeControls_programSpec } from './__generated__/IdeControls_programSpec.graphql';
import clsx from 'clsx';

const DEFAULT_STEP_INTERVAL = 1000; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [1, 2, 3];

const saveUserProgramMutation = graphql`
  mutation IdeControls_Mutation(
    $programSpecId: ID!
    $fileName: String!
    $sourceCode: String!
  ) {
    saveUserProgram(
      input: {
        programSpecId: $programSpecId
        fileName: $fileName
        sourceCode: $sourceCode
      }
    ) {
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
    padding: spacing(1),
    backgroundColor: palette.background.default,
  },
}));

/**
 * A component to edit and run GDLK programs.
 */
const IdeControls: React.FC<{
  className?: string;
  programSpec: IdeControls_programSpec;
  relay: RelayProp;
}> = ({ className, programSpec }) => {
  const localClasses = useLocalStyles();
  const { machineState, sourceCode, wsStatus, wsSend } = useContext(IdeContext);
  const [mutate, { loading: saveLoading }] = useMutation<IdeControls_Mutation>(
    saveUserProgramMutation
  );

  const [saveState, setSaveState] = useState<'success' | 'error' | undefined>();
  const [stepping, setStepping] = useState<boolean>(false);
  const [stepSpeed, setStepSpeed] = useState<number>(STEP_SPEED_OPTIONS[0]);
  const [lastCompiledCode, setLastCompiledCode] = useState<
    string | undefined
  >();
  const intervalId = useRef<number | undefined>();

  // Helper to send a step msg over the websocket
  const step = useCallback(() => wsSend({ type: 'step' }), [wsSend]);

  // Helper to cancel the current step interval
  const pause = useCallback(() => {
    if (intervalId.current !== undefined) {
      window.clearInterval(intervalId.current);
      intervalId.current = undefined;
    }
  }, [intervalId]);

  // Helper to initiate a step interval at some speed.
  const playAtSpeed = useCallback(
    (speed: number) => {
      pause();
      intervalId.current = window.setInterval(
        step,
        DEFAULT_STEP_INTERVAL / speed
      );
    },
    [intervalId, step, pause]
  );

  // If the play/pause state changes, or the play speed changes, we want to
  // update the play rate
  useEffect(() => {
    if (stepping) {
      playAtSpeed(stepSpeed);
    } else {
      pause();
    }
  }, [playAtSpeed, pause, step, stepping, stepSpeed]);

  const { userProgram } = programSpec;
  // This shouldn't be possible, but we need to appease the type checker
  if (!userProgram) {
    throw new Error('`programSpec.userProgram` should not be null');
  }

  return (
    <div className={clsx(localClasses.controls, className)}>
      <LoadingButton
        loading={saveLoading}
        // Disabled if there aren't any unsaved changes
        disabled={userProgram.sourceCode === sourceCode}
        onClick={() => {
          setSaveState(undefined);
          mutate({
            variables: {
              programSpecId: programSpec.id,
              fileName: userProgram.fileName,
              sourceCode,
            },
            onCompleted: () => setSaveState('success'),
            onError: () => setSaveState('error'),
          });
        }}
      >
        Save
      </LoadingButton>
      <Snackbar
        open={Boolean(saveState)}
        autoHideDuration={3000}
        onClose={() => setSaveState(undefined)}
      >
        {saveState === 'success' ? (
          <Alert severity="success">Solution saved</Alert>
        ) : (
          <Alert severity="error">Error saving program</Alert>
        )}
      </Snackbar>

      <Button
        disabled={wsStatus !== 'connected' || sourceCode === lastCompiledCode}
        onClick={() => {
          wsSend({ type: 'compile', content: { sourceCode } });
          setLastCompiledCode(sourceCode);
        }}
      >
        Compile
      </Button>
      <Button
        disabled={
          wsStatus !== 'connected' ||
          !machineState ||
          machineState.isComplete ||
          stepping
        }
        onClick={step}
      >
        Step
      </Button>

      <Button
        disabled={
          wsStatus !== 'connected' || !machineState || machineState.isComplete
        }
        onClick={() => setStepping((prev) => !prev)}
      >
        {stepping ? 'Pause' : 'Play'}
      </Button>

      <RadioGroup
        aria-label="program step speed"
        name="program step speed"
        row
        value={stepSpeed}
        onChange={(e) => setStepSpeed(parseInt(e.target.value, 10))}
      >
        {STEP_SPEED_OPTIONS.map((speed) => (
          <FormControlLabel
            key={speed}
            value={speed}
            control={<Radio />}
            label={`${speed}x`}
          />
        ))}
      </RadioGroup>
    </div>
  );
};

export default createFragmentContainer(IdeControls, {
  programSpec: graphql`
    fragment IdeControls_programSpec on ProgramSpecNode
      @argumentDefinitions(fileName: { type: "String!" }) {
      id
      userProgram(fileName: $fileName) {
        fileName
        sourceCode
      }
    }
  `,
});
