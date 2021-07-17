import React from 'react';
import HardwareSpecList from './HardwareSpecList';
import { Card, CardHeader, Typography } from '@material-ui/core';
import { hardware } from '@root/data/hardware';

const HardwareSpecListCard: React.FC = () => {
  return (
    <Card>
      <CardHeader
        title={<Typography variant="h2">Hardware Specs</Typography>}
      />
      <HardwareSpecList hardwareSpecs={Object.values(hardware)} />
    </Card>
  );
};

export default HardwareSpecListCard;
