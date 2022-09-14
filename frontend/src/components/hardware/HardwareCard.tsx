import React from "react";
import {
  Typography,
  Card,
  CardHeader,
  CardContent,
  Button,
} from "@mui/material";
import SimpleTable from "../common/SimpleTable";
import { useRecoilValue } from "recoil";
import { hardwareState } from "@root/state/user";
import { hardwareComponents } from "@root/data/hardware";
import useHardwareStore from "@root/hooks/useHardwareStore";
import { formatCurrency } from "@root/util/format";
import { makeStyles } from "@mui/styles";

const useLocalStyles = makeStyles({
  table: {
    width: "100%",
  },
});

/**
 * Show details on the user's current hardware capabilities
 */
const HardwareCard: React.FC = () => {
  const localClasses = useLocalStyles();
  const hardware = useRecoilValue(hardwareState);
  const { getUpgradeCost, canUpgrade, purchaseUpgrade } = useHardwareStore();

  return (
    <Card sx={{ maxWidth: 400 }}>
      <CardHeader title={<Typography variant="h2">Hardware</Typography>} />
      <CardContent>
        <SimpleTable
          className={localClasses.table}
          data={hardwareComponents.map(({ component, label }) => ({
            label,
            value: hardware[component],
            // If editing, show an upgrade button
            action: (
              <Button
                disabled={!canUpgrade(component)}
                onClick={() => purchaseUpgrade(component)}
              >
                Upgrade ({formatCurrency(getUpgradeCost(component))})
              </Button>
            ),
          }))}
        />
      </CardContent>
    </Card>
  );
};

export default HardwareCard;
