import React, {
  useCallback,
  useState,
  useContext,
  useEffect,
  useRef,
} from 'react';
import { makeStyles } from '@material-ui/core';
import {
  Pause as IconPause,
  PlayArrow as IconPlayArrow,
  Refresh as IconRefresh,
  NavigateNext as IconNavigateNext,
  SkipNext as IconSkipNext,
} from '@material-ui/icons';
import { ToggleButton, ToggleButtonGroup } from '@material-ui/lab';
import { IdeContext } from 'state/ide';
import clsx from 'clsx';
import IconButton from 'components/common/IconButton';

const DEFAULT_STEP_INTERVAL = 1000; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [2, 20];

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
}> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { compiledState, execute, reset } = useContext(IdeContext);
  // We use this a few times so let's store it here
  const executeNext = useCallback(() => execute(false), [execute]);

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
          title="Execute to End"
          disabled={!machineState || machineState.terminated || stepping}
          onClick={() => execute(true)}
        >
          <IconSkipNext />
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
        onChange={(e, newStepSpeed) => {
          // Prevent de-selecting
          if (newStepSpeed !== null) {
            setStepSpeed(newStepSpeed);
          }
        }}
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
    </div>
  );
};

export default IdeControls;
