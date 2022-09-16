import { hardwareComponentsByName } from "@root/data/hardware";
import { currencyState, hardwareUpgradeState } from "@root/state/user";
import { assertIsDefined, isDefined } from "@root/util/guards";
import { HardwareComponent, HardwareComponentUpgrade } from "@root/util/types";
import { useRecoilState } from "recoil";

interface ReturnValue {
  getNextUpgrade: (
    component: HardwareComponent
  ) => HardwareComponentUpgrade | undefined;
  canUpgrade: (component: HardwareComponent) => boolean;
  purchaseUpgrade: (component: HardwareComponent) => void;
}

/**
 * TODO
 */
function useHardwareStore(): ReturnValue {
  const [currency, setCurrency] = useRecoilState(currencyState);
  const [hardwareUpgrades, setHardwareUpgrades] =
    useRecoilState(hardwareUpgradeState);

  const getNextUpgrade: ReturnValue["getNextUpgrade"] = (component) => {
    const { upgrades } = hardwareComponentsByName[component];
    const appliedUpgrades = hardwareUpgrades[component];
    // Grab the next upgrade
    if (appliedUpgrades < upgrades.length) {
      return upgrades[appliedUpgrades];
    }
    // No mroe upgrades left
    return undefined;
  };

  const canUpgrade: ReturnValue["canUpgrade"] = (component) => {
    const nextUpgrade = getNextUpgrade(component);
    // Check that we can afford the next upgrade. If no upgrade is available,
    // then obviously we can't upgrade
    return isDefined(nextUpgrade) && nextUpgrade.cost <= currency;
  };

  const purchaseUpgrade: ReturnValue["purchaseUpgrade"] = (component) => {
    const nextUpgrade = getNextUpgrade(component);
    // This function shouldn't every be called if no upgrade is available
    assertIsDefined(nextUpgrade, "No upgrade available");
    setCurrency((old) => old - nextUpgrade?.cost);
    setHardwareUpgrades((old) => ({
      ...old,
      [component]: old[component] + 1,
    }));
  };

  return {
    getNextUpgrade,
    canUpgrade,
    purchaseUpgrade,
  };
}

export default useHardwareStore;
