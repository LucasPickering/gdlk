import { hardwareComponentsByName } from "@root/data/hardware";
import { currencyState, hardwareState } from "@root/state/user";
import { Currency, HardwareComponent } from "@root/util/types";
import { useRecoilState } from "recoil";

interface ReturnValue {
  getUpgradeCost: (component: HardwareComponent) => Currency;
  canUpgrade: (component: HardwareComponent) => boolean;
  purchaseUpgrade: (component: HardwareComponent) => void;
}

/**
 * TODO
 */
function useHardwareStore(): ReturnValue {
  const [currency, setCurrency] = useRecoilState(currencyState);
  const [hardware, setHardware] = useRecoilState(hardwareState);

  const getUpgradeCost: ReturnValue["getUpgradeCost"] = (component) => {
    const { min, upgradeCostFactor } = hardwareComponentsByName[component];
    const previousUpgrades = hardware[component] - min;
    return (previousUpgrades + 1) * upgradeCostFactor;
  };

  const canUpgrade: ReturnValue["canUpgrade"] = (component) => {
    const currentValue = hardware[component];
    const { max } = hardwareComponentsByName[component];
    return currentValue < max && getUpgradeCost(component) <= currency;
  };

  const purchaseUpgrade: ReturnValue["purchaseUpgrade"] = (component) => {
    const cost = getUpgradeCost(component);
    setCurrency((old) => old - cost);
    setHardware((old) => ({ ...old, [component]: old[component] + 1 }));
  };

  return {
    getUpgradeCost,
    canUpgrade,
    purchaseUpgrade,
  };
}

export default useHardwareStore;
