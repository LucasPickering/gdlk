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

export interface PuzzleSolution {
  sourceCode: string;
}

/**
 * All of a user's solutions, to all puzzles
 */
export interface PuzzleSolutions {
  [puzzleSlug: string]: PuzzleSolution;
}
