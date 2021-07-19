import React from 'react';
import { ListItem, ListItemText } from '@material-ui/core';
import UnstyledLink from '@root/components/common/UnstyledLink';
import { HardwareSpec } from '@root/util/types';

const HardwareSpecListItem: React.FC<{
  hardwareSpec: HardwareSpec;
}> = ({ hardwareSpec }) => (
  <ListItem
    key={hardwareSpec.name}
    button
    component={UnstyledLink}
    to={`/hardware/${hardwareSpec.slug}`}
  >
    <ListItemText primary={hardwareSpec.name} />
  </ListItem>
);

export default HardwareSpecListItem;
