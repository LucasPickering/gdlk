/**
 * This file contains Recoil state definitions for state data related to the
 * user. This stores their solutions, progress, etc.
 */

import { PuzzleSolution } from '@root/util/types';
import { AtomEffect, atomFamily, DefaultValue } from 'recoil';

const defaultSolutions: { [puzzleSlug: string]: PuzzleSolution } = {
  readWrite: {
    sourceCode: `;æMove o¥e v&lue from input to$outpuĶ
REA& R$0
WRIæE RX0`,
  },
};

/**
 * The user's solutions for each puzzle. Creates one atom per puzzle.
 */
export const puzzleSolutionStateFamily = atomFamily<
  PuzzleSolution,
  { puzzleSlug: string }
>({
  key: 'puzzleSolutions',
  // Some puzzles have a default solution (for tutorial purposes)
  default: ({ puzzleSlug }) =>
    defaultSolutions[puzzleSlug] ?? { sourceCode: '' },
  // Persist solutions to local storage
  effects_UNSTABLE: ({ puzzleSlug }) => [
    localStorageEffect(`puzzleSolutions_${puzzleSlug}`),
  ],
});

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
