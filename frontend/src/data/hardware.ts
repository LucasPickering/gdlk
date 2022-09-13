import { HardwareComponentMetadata, HardwareComponent } from "@root/util/types";

/**
 * TODO
 */
export const hardwareComponents: HardwareComponentMetadata[] = [
  {
    component: "numRegisters",
    label: "Registers",
    min: 1,
    max: 8,
    upgradeCostFactor: 10,
  },
  {
    component: "numStacks",
    label: "Stacks",
    min: 0,
    max: 4,
    upgradeCostFactor: 20,
  },
  {
    component: "maxStackLength",
    label: "Stack Size",
    min: 0,
    max: 16,
    upgradeCostFactor: 4,
  },
];

export const hardwareComponentsByName: Record<
  HardwareComponent,
  HardwareComponentMetadata
> = hardwareComponents.reduce((acc, component) => {
  acc[component.component] = component;
  return acc;
}, {} as Record<HardwareComponent, HardwareComponentMetadata>);
