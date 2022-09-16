import React, {
  useCallback,
  useState,
  useContext,
  useEffect,
  useRef,
} from "react";
import {
  Pause as IconPause,
  PlayArrow as IconPlayArrow,
  Refresh as IconRefresh,
  NavigateNext as IconNavigateNext,
  SkipNext as IconSkipNext,
} from "@mui/icons-material";
import { Stack, ToggleButton, ToggleButtonGroup } from "@mui/material";
import { IdeContext } from "@root/state/ide";
import IconButton from "@root/components/common/IconButton";

const DEFAULT_STEP_INTERVAL = 1000; // ms between steps at 1x speed
const STEP_SPEED_OPTIONS: number[] = [2, 20];

/**
 * A component to edit and run GDLK programs.
 */
const IdeControls: React.FC<{
  className?: string;
}> = ({ className }) => {
  const { compiledState, stepping, setStepping, execute, reset } =
    useContext(IdeContext);
  // We use this a few times so let's store it here
  const executeNext = useCallback(() => execute(false), [execute]);

  const [stepSpeed, setStepSpeed] = useState<number>(STEP_SPEED_OPTIONS[0]);
  const intervalIdRef = useRef<number | undefined>();

  const machineState =
    compiledState?.type === "compiled" ? compiledState.machineState : undefined;

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
  }, [terminated, setStepping]);

  return (
    <Stack className={className} direction="row" padding={1}>
      <IconButton
        title="Execute Next Instruction"
        disabled={!machineState || machineState.terminated || stepping}
        onClick={executeNext}
        size="large"
      >
        <IconNavigateNext />
      </IconButton>

      <IconButton
        title={stepping ? "Pause Execution" : "Execute Program"}
        disabled={!machineState || machineState.terminated}
        onClick={() => setStepping((prev) => !prev)}
        size="large"
      >
        {stepping ? <IconPause /> : <IconPlayArrow />}
      </IconButton>

      <IconButton
        title="Execute to End"
        disabled={!machineState || machineState.terminated || stepping}
        onClick={() => execute(true)}
        size="large"
      >
        <IconSkipNext />
      </IconButton>

      <IconButton
        title={"Reset Program"}
        // Disable if the program hasn't started yet
        disabled={!machineState || machineState.cycleCount === 0}
        onClick={() => {
          reset();
          setStepping(false);
        }}
        size="large"
      >
        <IconRefresh />
      </IconButton>

      <ToggleButtonGroup
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
            key={speed}
            value={speed}
            aria-label={`${speed} times speed`}
            // Consistent sizing
            sx={{ width: 40 }}
          >
            {">".repeat(i + 1)}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </Stack>
  );
};

export default IdeControls;
