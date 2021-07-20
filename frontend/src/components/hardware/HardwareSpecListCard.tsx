import React from 'react';
import {
  Card,
  CardContent,
  CardHeader,
  List,
  ListItem,
  ListItemText,
  makeStyles,
  Typography,
} from '@material-ui/core';
import { hardware } from '@root/data/hardware';
import UnstyledLink from '../common/UnstyledLink';

const useLocalStyles = makeStyles({
  hardwareSpecList: {
    // Just rely on the card's padding
    padding: 0,
  },
});

const HardwareSpecListCard: React.FC = () => {
  const localClasses = useLocalStyles();

  return (
    <Card>
      <CardHeader
        title={<Typography variant="h2">Hardware Specs</Typography>}
      />
      <CardContent>
        <List className={localClasses.hardwareSpecList} dense>
          {Object.values(hardware).map((hardwareSpec) => (
            <ListItem
              key={hardwareSpec.name}
              button
              component={UnstyledLink}
              to={`/hardware/${hardwareSpec.slug}`}
            >
              <ListItemText primary={hardwareSpec.name} />
            </ListItem>
          ))}
        </List>
      </CardContent>
    </Card>
  );
};

export default HardwareSpecListCard;
