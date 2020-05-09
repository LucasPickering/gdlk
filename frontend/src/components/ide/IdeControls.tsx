import React, { useEffect, useState, useContext } from 'react';
import graphql from 'babel-plugin-relay/macro';
import {
  makeStyles,
  Snackbar,
  FormControlLabel,
  RadioGroup,
  Radio,
} from '@material-ui/core';
import {
  Build as IconBuild,
  Replay as IconReplay,
  Pause as IconPause,
  PlayArrow as IconPlayArrow,
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

const DEFAULT_STEP_INTERVAL = 500; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [1, 5, 10];

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
  const { machineState, sourceCode, wsStatus, wsSend } = useContext(IdeContext);
  const [mutate, { loading: saveLoading }] = useMutation<IdeControls_Mutation>(
    saveUserProgramMutation
  );

  const [saveState, setSaveState] = useState<'success' | 'error' | undefined>();
  const [stepping, setStepping] = useState<boolean>(false);
  const [stepSpeed, setStepSpeed] = useState<number>(STEP_SPEED_OPTIONS[0]);

  const wsConnected = wsStatus === 'connected';

  // Effect to start/stop the auto-stepper
  useEffect(() => {
    if (wsConnected) {
      if (stepping) {
        wsSend({
          type: 'autoStepStart',
          content: {
            interval: Math.round(DEFAULT_STEP_INTERVAL / stepSpeed),
          },
        });
      } else {
        wsSend({ type: 'autoStepStop' });
      }
    }
  }, [wsConnected, wsSend, stepping, stepSpeed]);

  const { userProgram } = programSpec;
  // This shouldn't be possible, but we need to appease the type checker
  if (!userProgram) {
    throw new Error('`programSpec.userProgram` should not be null');
  }

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
          title={machineState ? 'Reset' : 'Compile'}
          disabled={!wsConnected}
          onClick={() => {
            setStepping(false);
            wsSend({ type: 'compile', content: { sourceCode } });
          }}
        >
          {machineState ? <IconReplay /> : <IconBuild />}
        </IconButton>

        <IconButton
          title="Execute Next Instruction"
          disabled={
            !wsConnected || !machineState || machineState.isComplete || stepping
          }
          onClick={() => wsSend({ type: 'step' })}
        >
          <IconChevronRight />
        </IconButton>

        <IconButton
          title={stepping ? 'Pause Execution' : 'Execute Program'}
          disabled={!wsConnected || !machineState || machineState.isComplete}
          onClick={() => setStepping((prev) => !prev)}
        >
          {stepping ? <IconPause /> : <IconPlayArrow />}
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
