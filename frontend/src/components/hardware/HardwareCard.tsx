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
  const { getNextUpgrade, canUpgrade, purchaseUpgrade } = useHardwareStore();

  return (
    <Card sx={{ maxWidth: 400 }}>
      <CardHeader title={<Typography variant="h2">Hardware</Typography>} />
      <CardContent>
        <SimpleTable
          className={localClasses.table}
          data={hardwareComponents.map(({ component, label }) => {
            const nextUpgrade = getNextUpgrade(component);
            return {
              label,
              value: hardware[component],
              // Show an upgrade button if one is available (even if it's not
              // affordable yet.)
              // TODO figure out some placeholder once a component is maxed
              action: nextUpgrade && (
                <Button
                  disabled={!canUpgrade(component)}
                  onClick={() => purchaseUpgrade(component)}
                >
                  Upgrade ({formatCurrency(nextUpgrade.cost)})
                </Button>
              ),
            };
          })}
        />
      </CardContent>
    </Card>
  );
};

export default HardwareCard;
