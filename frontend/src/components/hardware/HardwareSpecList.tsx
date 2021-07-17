import React from 'react';
import { CardContent, List, makeStyles } from '@material-ui/core';
import HardwareSpecListItem from './HardwareSpecListItem';
import { HardwareSpec } from '@root/util/types';

const useLocalStyles = makeStyles(() => ({
  hardwareSpecList: {
    // Just rely on the card's padding
    padding: 0,
  },
}));

const HardwareSpecList: React.FC<{
  hardwareSpecs: HardwareSpec[];
}> = ({ hardwareSpecs }) => {
  const localClasses = useLocalStyles();

  return (
    <CardContent>
      <List className={localClasses.hardwareSpecList} dense>
        {hardwareSpecs.map((hardwareSpec) => (
          <HardwareSpecListItem
            key={hardwareSpec.name}
            hardwareSpec={hardwareSpec}
          />
        ))}
      </List>
    </CardContent>
  );
};

export default HardwareSpecList;
