/**
 * The currency that the player earns and spends.
 */
export type Currency = number;

export interface HardwareSpec {
  numRegisters: number;
  numStacks: number;
  maxStackLength: number;
}

export interface Puzzle {
  name: string;
  slug: string;
  description: string;
  currencyValue: Currency;
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
