import React, { useState } from 'react';
import {
  Typography,
  Card,
  CardHeader,
  CardContent,
  IconButton,
} from '@material-ui/core';
import {
  Add as IconAdd,
  Remove as IconRemove,
  Done as IconDone,
  Edit as IconEdit,
} from '@material-ui/icons';
import { hardwareSpecState } from '@root/state/user';
import { useRecoilState } from 'recoil';
import SimpleTable from '../common/SimpleTable';
import { HardwareSpec } from '@root/util/types';

/**
 * Show details on the user's current hardware capabilities
 */
const HardwareSpecCard: React.FC = () => {
  const [editing, setEditing] = useState<boolean>(false);

  return (
    <Card>
      <CardHeader
        title={<Typography variant="h2">Hardware</Typography>}
        action={
          <IconButton onClick={() => setEditing((old) => !old)}>
            {editing ? <IconDone /> : <IconEdit />}
          </IconButton>
        }
      />
      <CardContent>
        <SimpleTable
          data={[
            {
              label: 'Registers',
              value: (
                <HardwareSpecField
                  field="numRegisters"
                  editing={editing}
                  min={1}
                  max={8}
                />
              ),
            },
            {
              label: 'Stacks',
              value: (
                <HardwareSpecField
                  field="numStacks"
                  editing={editing}
                  min={0}
                  max={4}
                />
              ),
            },
            {
              label: 'Stack Size',
              value: (
                <HardwareSpecField
                  field="maxStackLength"
                  editing={editing}
                  min={0}
                  max={16}
                />
              ),
            },
          ]}
        />
      </CardContent>
    </Card>
  );
};

const HardwareSpecField: React.FC<{
  field: keyof HardwareSpec;
  editing: boolean;
  min: number;
  max: number;
}> = ({ field, editing, min, max }) => {
  const [hardwareSpec, setHardwareSpec] = useRecoilState(hardwareSpecState);
  const value = hardwareSpec[field];

  if (!editing) {
    return <>{value}</>;
  }

  return (
    <>
      <IconButton
        disabled={value <= min}
        onClick={() =>
          setHardwareSpec((old) => ({
            ...old,
            [field]: old[field] - 1,
          }))
        }
      >
        <IconRemove />
      </IconButton>
      {value}
      <IconButton
        disabled={value >= max}
        onClick={() =>
          setHardwareSpec((old) => ({
            ...old,
            [field]: old[field] + 1,
          }))
        }
      >
        <IconAdd />
      </IconButton>
    </>
  );
};

export default HardwareSpecCard;
