import { HardwareComponentMetadata, HardwareComponent } from "@root/util/types";

/**
 * TODO
 */
export const hardwareComponents: HardwareComponentMetadata[] = [
  {
    component: "numRegisters",
    label: "Registers",
    default: 1,
    upgrades: [
      { increase: 1, cost: 10 },
      { increase: 1, cost: 20 },
      { increase: 1, cost: 40 },
    ],
  },
  {
    component: "numStacks",
    label: "Stacks",
    default: 0,
    upgrades: [
      { increase: 1, cost: 20 },
      { increase: 1, cost: 40 },
      { increase: 1, cost: 80 },
    ],
  },
  {
    component: "maxStackLength",
    label: "Stack Size",
    default: 0,
    upgrades: [
      { increase: 4, cost: 20 },
      { increase: 4, cost: 40 },
      { increase: 8, cost: 80 },
    ],
  },
];

export const hardwareComponentsByName: Record<
  HardwareComponent,
  HardwareComponentMetadata
> = hardwareComponents.reduce((acc, component) => {
  acc[component.component] = component;
  return acc;
}, {} as Record<HardwareComponent, HardwareComponentMetadata>);
