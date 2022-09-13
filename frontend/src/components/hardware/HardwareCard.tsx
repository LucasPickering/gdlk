import React from "react";
import {
  Typography,
  Card,
  CardHeader,
  CardContent,
  Button,
  makeStyles,
} from "@material-ui/core";
import SimpleTable from "../common/SimpleTable";
import { useRecoilValue } from "recoil";
import { currencyState, hardwareState } from "@root/state/user";
import { hardwareComponents } from "@root/data/hardware";
import useHardwareStore from "@root/hooks/useHardwareStore";

const useLocalStyles = makeStyles({
  card: {
    maxWidth: 400,
  },
  table: {
    width: "100%",
  },
});

/**
 * Show details on the user's current hardware capabilities
 */
const HardwareCard: React.FC = () => {
  const localClasses = useLocalStyles();
  const currency = useRecoilValue(currencyState);
  const hardware = useRecoilValue(hardwareState);
  const { getUpgradeCost, canUpgrade, purchaseUpgrade } = useHardwareStore();

  return (
    <Card className={localClasses.card}>
      <CardHeader
        title={<Typography variant="h2">Hardware</Typography>}
        subheader={currency.toString()}
      />
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
                Upgrade ({getUpgradeCost(component).toString()})
              </Button>
            ),
          }))}
        />
      </CardContent>
    </Card>
  );
};

export default HardwareCard;
