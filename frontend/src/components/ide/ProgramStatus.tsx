import React, { useContext } from 'react';
import { IdeContext } from '@root/state/ide';
import clsx from 'clsx';
import { makeStyles } from '@material-ui/core';
import { MachineState } from '@root/util/compile';

const useLocalStyles = makeStyles(({ spacing }) => ({
  programStatus: {
    padding: spacing(1),
  },
}));

function getStatusText(machineState: MachineState): string {
  if (!machineState.terminated) {
    return '...';
  }

  if (machineState.successful) {
    return 'SUCCESS';
  }

  // TODO show failure reason
  return 'FAILURE';
}

const ProgramStatus: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { compiledState } = useContext(IdeContext);

  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  return (
    <div className={clsx(className, localClasses.programStatus)}>
      <div>CPU Cycles: {machineState?.cycleCount ?? '–'}</div>
      {machineState && <div>{getStatusText(machineState)}</div>}
    </div>
  );
};

export default ProgramStatus;
