export interface HardwareSpec {
  name: string;
  slug: string;
  numRegisters: number;
  numStacks: number;
  maxStackLength: number;
}

export interface Puzzle {
  name: string;
  slug: string;
  description: string;
  examples: Array<{ input: number[]; output: number[] }>;
  input: number[];
  expectedOutput: number[];
}
