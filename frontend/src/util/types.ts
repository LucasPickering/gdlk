/**
 * A value of currency, earned via puzzles and used to buy upgrades.
 */
export type Currency = number;

export type HardwareComponent = "numRegisters" | "numStacks" | "maxStackLength";

/**
 * One piece of a computer. Each component can be independently upgraded by
 * the user, and has a different purpose in program execution.
 */
export interface HardwareComponentMetadata {
  component: HardwareComponent;
  label: string;
  default: number;
  upgrades: Array<HardwareComponentUpgrade>;
}

/**
 * A single available upgrade for a hardware component. Defines how much it
 * costs, as well as the *gained value* of the upgrade. I.e. to get the new
 * component value, add the upgrade value to its current value.
 */
export interface HardwareComponentUpgrade {
  increase: number;
  cost: Currency;
}

/**
 * A computer used to execute programs. The user always has one hardware at
 * a time, and can upgrade it by spending currency.
 */
export type Hardware = Record<HardwareComponent, number>;

export interface Puzzle {
  name: string;
  slug: string;
  description: string;
  reward: Currency;
  examples: Array<{ input: number[]; output: number[] }>;
  input: number[];
  expectedOutput: number[];
}

/**
 * Data+metadata on a user's solution to a particular puzzle
 */
export interface PuzzleSolution {
  sourceCode: string;
  solved: boolean;
}

/**
 * A user's completion level for a particular puzzle:
 * - locked: they can't access it yet (need to complete prereqs)
 * - unlocked: accessible but unsolved
 * - solved: completed *at some point*. If a puzzle has been solved once, it
 *  will always be tagged as solved, even if they delete the solution
 */
export type PuzzleCompletion = "locked" | "unlocked" | "solved";
