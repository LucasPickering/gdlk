/**
 * This file contains Recoil state definitions for state data related to the
 * user. This stores their solutions, progress, etc.
 */

import { AtomEffect, atomFamily, DefaultValue, selectorFamily } from 'recoil';

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
export type PuzzleCompletion = 'locked' | 'unlocked' | 'solved';

/**
 * The user's solutions for each puzzle. Creates one atom per puzzle.
 */
export const puzzleSolutionStateFamily = atomFamily<
  PuzzleSolution,
  { puzzleSlug: string }
>({
  key: 'puzzleSolutions',
  default: ({ puzzleSlug }) => ({
    sourceCode: defaultSolutionSourceCode[puzzleSlug] ?? '',
    solved: false,
  }),
  // Persist solutions to local storage
  effects_UNSTABLE: ({ puzzleSlug }) => [
    localStorageEffect(`puzzleSolutions_${puzzleSlug}`),
  ],
});

/**
 * Selector for the user's completion level on a particular puzzle
 */
export const puzzleCompletionState = selectorFamily<
  PuzzleCompletion,
  { puzzleSlug: string }
>({
  key: 'puzzleCompletion',
  get:
    (param) =>
    ({ get }) => {
      const { solved } = get(puzzleSolutionStateFamily(param));
      if (solved) {
        return 'solved';
      }
      // TODO implement puzzle locking
      return 'unlocked';
    },
});

/**
 * Some puzzles have a default solution, for tutorial purposes
 */
const defaultSolutionSourceCode: { [puzzleSlug: string]: string } = {
  readWrite: `;æMove o¥e v&lue from input to$outpuĶ
REA& R$0
WRIæE RX0`,
};

function localStorageEffect(key: string): AtomEffect<PuzzleSolution> {
  return ({ setSelf, onSet }) => {
    const savedValue = localStorage.getItem(key);
    if (savedValue != null) {
      setSelf(JSON.parse(savedValue));
    }

    onSet((newValue) => {
      if (newValue instanceof DefaultValue) {
        localStorage.removeItem(key);
      } else {
        localStorage.setItem(key, JSON.stringify(newValue));
      }
    });
  };
}
