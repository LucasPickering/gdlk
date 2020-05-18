import React, { useState, useContext, useEffect, useRef } from 'react';
import graphql from 'babel-plugin-relay/macro';
import {
  makeStyles,
  Snackbar,
  FormControlLabel,
  RadioGroup,
  Radio,
} from '@material-ui/core';
import {
  Pause as IconPause,
  PlayArrow as IconPlayArrow,
  Refresh as IconRefresh,
  ChevronRight as IconChevronRight,
  Save as IconSave,
} from '@material-ui/icons';
import { Alert } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import { useMutation } from 'relay-hooks';
import { IdeControls_Mutation } from './__generated__/IdeControls_Mutation.graphql';
import { createFragmentContainer, RelayProp } from 'react-relay';
import { IdeControls_programSpec } from './__generated__/IdeControls_programSpec.graphql';
import clsx from 'clsx';
import IconButton from 'components/common/IconButton';
import { assertIsDefined } from 'util/guards';

const DEFAULT_STEP_INTERVAL = 1000; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [1, 5, 20];

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

    backgroundColor: palette.background.default,
  },
  buttons: {
    padding: spacing(1),
  },
  speedSelect: {
    padding: spacing(1),
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

  const { userProgram } = programSpec;
  assertIsDefined(userProgram);
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

  // When the program completes, stop the stepper
  const isComplete = Boolean(machineState?.isComplete);
  useEffect(() => {
    if (isComplete) {
      window.clearInterval(intervalIdRef.current);
      setStepping(false);
    }
  }, [isComplete]);

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
                programSpecId: programSpec.id,
                fileName: userProgram.fileName,
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
          disabled={!machineState || machineState.isComplete || stepping}
          onClick={executeNext}
        >
          <IconChevronRight />
        </IconButton>

        <IconButton
          title={stepping ? 'Pause Execution' : 'Execute Program'}
          disabled={!machineState || machineState.isComplete}
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

      <RadioGroup
        classes={{ root: localClasses.speedSelect }}
        name="Execution Speed"
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
