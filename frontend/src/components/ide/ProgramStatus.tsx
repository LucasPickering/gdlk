import React, { useContext } from 'react';
import { IdeContext } from 'state/ide';
import clsx from 'clsx';
import { makeStyles } from '@material-ui/core';

const useLocalStyles = makeStyles(({ spacing }) => ({
  programStatus: {
    padding: spacing(1),
  },
}));

const ProgramStatus: React.FC<{ className?: string }> = ({ className }) => {
  const localClasses = useLocalStyles();
  const { compiledState } = useContext(IdeContext);

  const machineState =
    compiledState?.type === 'compiled' ? compiledState.machineState : undefined;

  return (
    <div className={clsx(className, localClasses.programStatus)}>
      <div>CPU Cycles: {machineState?.cycleCount ?? '-'}</div>
      {machineState?.terminated && (
        <div>{machineState?.successful ? 'SUCCESS' : 'FAILURE'}</div>
      )}
    </div>
  );
};

export default ProgramStatus;
