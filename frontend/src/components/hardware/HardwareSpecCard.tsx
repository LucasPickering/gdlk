import React, { useState } from "react";
import {
  Typography,
  Card,
  CardHeader,
  CardContent,
  IconButton,
} from "@material-ui/core";
import { Done as IconDone, Edit as IconEdit } from "@material-ui/icons";
import SimpleTable from "../common/SimpleTable";
import HardwareSpecField from "./HardwareSpecField";

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
              label: "Registers",
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
              label: "Stacks",
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
              label: "Stack Size",
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

export default HardwareSpecCard;
