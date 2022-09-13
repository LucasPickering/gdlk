/**
 * This file contains Recoil state definitions for state data related to the
 * user. This stores their solutions, progress, etc.
 */

import {
  Currency,
  HardwareSpec,
  PuzzleCompletion,
  PuzzleSolution,
} from "@root/util/types";
import { atom, AtomEffect, atomFamily, selectorFamily } from "recoil";

/**
 * Track how much money the user has
 */
export const currencyState = atom<Currency>({
  key: "currency",
  default: 0,
  effects: [localStorageEffect("currency")],
});

/**
 * Track the user's current hardware capabilities, which can be upgraded over
 * time
 */
export const hardwareSpecState = atom<HardwareSpec>({
  key: "hardwareSpec",
  default: {
    numRegisters: 2,
    numStacks: 0,
    maxStackLength: 0,
  },
  effects: [localStorageEffect("hardwareSpec")],
});

/**
 * The user's solutions for each puzzle. Creates one atom per puzzle.
 */
export const puzzleSolutionStateFamily = atomFamily<
  PuzzleSolution,
  { puzzleSlug: string }
>({
  key: "puzzleSolutions",
  default: ({ puzzleSlug }) => ({
    sourceCode: defaultSolutionSourceCode[puzzleSlug] ?? "",
    solved: false,
  }),
  // Persist solutions to local storage
  effects: ({ puzzleSlug }) => [
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
  key: "puzzleCompletion",
  get:
    (param) =>
    ({ get }) => {
      const { solved } = get(puzzleSolutionStateFamily(param));
      if (solved) {
        return "solved";
      }
      // TODO implement puzzle locking
      return "unlocked";
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

/**
 * An effect that stores app state in local storage, via Recoil
 */
function localStorageEffect<T>(key: string): AtomEffect<T> {
  return ({ setSelf, onSet }) => {
    const savedValue = localStorage.getItem(key);
    if (savedValue != null) {
      // We kinda have to do a trust fall here and just hope that the parsed
      // value has the correct type
      setSelf(JSON.parse(savedValue));
    }

    onSet((newValue, _, isReset) => {
      if (isReset) {
        localStorage.removeItem(key);
      } else {
        localStorage.setItem(key, JSON.stringify(newValue));
      }
    });
  };
}
