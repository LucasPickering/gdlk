import { HardwareSpec } from '@root/util/types';

export const hardware: Record<string, HardwareSpec> = {
  // TODO de-dupe key+slug
  k200: {
    name: 'K200',
    slug: 'k200',
    numRegisters: 2,
    numStacks: 0,
    maxStackLength: 0,
  },
};
