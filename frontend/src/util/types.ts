/**
 * The currency that the player earns and spends. We use a class instead of just
 * a type alias so we can get newtype-like benefits, and to enforce consistent
 * formatting.
 */
export class Currency {
  constructor(private readonly currencyValue: number) {}

  public plus(other: Currency): Currency {
    return new Currency(this.currencyValue + other.currencyValue);
  }

  public minus(other: Currency): Currency {
    return new Currency(this.currencyValue - other.currencyValue);
  }

  public toString(): string {
    return `${this.currencyValue}ƒ`;
  }
}

export type HardwareComponent = "numRegisters" | "numStacks" | "maxStackLength";

/**
 * One piece of a computer. Each component can be independently upgraded by
 * the user, and has a different purpose in program execution.
 */
export interface HardwareComponentMetadata {
  component: HardwareComponent;
  label: string;
  min: number;
  max: number;
  upgradeCostFactor: number;
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
