/**
 * This file contains Recoil state definitions for state data related to the
 * user. This stores their solutions, progress, etc.
 */

import { hardwareComponentsByName } from "@root/data/hardware";
import { mapValues, sum } from "@root/util/general";
import { isDefined } from "@root/util/guards";
import { StorageHandler } from "@root/util/storage";
import {
  Currency,
  Hardware,
  PuzzleCompletion,
  PuzzleSolution,
} from "@root/util/types";
import { atom, AtomEffect, atomFamily, selector, selectorFamily } from "recoil";

/**
 * Track how much money the user has
 */
export const currencyState = atom<Currency>({
  key: "currency",
  default: 0,
  effects: [localStorageEffect("currency")],
});

/**
 * Track the user's current hardware *upgrades*, which can be purchased over
 * time. Each of the inner fields is the number of *upgrades* applied to that
 * component.
 */
export const hardwareUpgradeState = atom<Hardware>({
  key: "hardwareUpgrade",
  default: {
    numRegisters: 0,
    numStacks: 0,
    maxStackLength: 0,
  },
  effects: [localStorageEffect("hardwareUpgrade")],
});

/**
 * TODO
 */
export const hardwareState = selector<Hardware>({
  key: "hardware",
  get: ({ get }) => {
    const hardwareUpgrades = get(hardwareUpgradeState);
    // For each component, compute its current value by compiling the upgrade
    // list as far as the user has purchased.
    // This feels kinda dumb, but blern always said to maintain minimal state
    // and derive the rest, so I'm going with that. It'll make it easier to
    // push out changes to the upgrade tiers anyway.
    return mapValues<Hardware, number>(
      hardwareUpgrades,
      (numUpgrades, componentName) => {
        const componentSpec = hardwareComponentsByName[componentName];
        return (
          componentSpec.default +
          sum(
            componentSpec.upgrades.slice(0, numUpgrades),
            (upgrade) => upgrade.increase
          )
        );
      }
    );
  },
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
    const storage = new StorageHandler<T>(key); // Use a wrapper class
    const value = storage.get();
    if (isDefined(value)) {
      // We kinda have to do a trust fall here and just hope that the parsed
      // value has the correct type
      setSelf(value.value);
    }

    onSet((newValue, _, isReset) => {
      if (isReset) {
        storage.delete();
      } else {
        storage.set(newValue);
      }
    });
  };
}
