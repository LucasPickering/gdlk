/**
 * This file contains context definitions for all of a user's persisted data.
 * Each category of data is broken into a separate context, and the context
 * may also contain functions to mutating the state. State values in here should
 * never be mutated directly! We need to use setters to trigger a re-render in
 * React.
 */

import { PuzzleSolution } from '@root/util/types';
import React from 'react';

export interface PuzzleSolutionsContextType {
  getPuzzleSolutions: (puzzleSlug: string) => PuzzleSolution[];
  getPuzzleSolution: (
    puzzleSlug: string,
    fileName: string
  ) => PuzzleSolution | undefined;
  createSolution: (puzzleSlug: string, fileName: string) => void;
  copySolution: (puzzleSlug: string, fileName: string) => void;
  renameSolution: (
    puzzleSlug: string,
    oldFileName: string,
    newFileName: string
  ) => void;
  deleteSolution: (puzzleSlug: string, fileName: string) => void;
  setSolutionSourceCode: (
    puzzleSlug: string,
    fileName: string,
    sourceCode: string
  ) => void;
}

export const PuzzleSolutionsContext =
  React.createContext<PuzzleSolutionsContextType>(
    {} as PuzzleSolutionsContextType
  );
