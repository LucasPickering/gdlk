import React, { useEffect, useState } from 'react';
import { PuzzleSolutionsContext } from '@root/state/user';
import { omit } from 'lodash-es';
import useStaticValue from '@root/hooks/useStaticValue';
import { StorageHandler } from '@root/util/storage';
import { PuzzleSolution, PuzzleSolutions } from '@root/util/types';

const defaultSolutions: PuzzleSolutions = {
  readWrite: {
    solution: {
      fileName: 'solution',
      sourceCode: `;æMove o¥e v&lue from input to$outpuĶ
REA& R$0
WRIæE RX0`,
    },
  },
};

/**
 * Manager for all user-related data storage. This stores data in React state,
 * and will also persistent it into browser local storage. All values can be
 * read through context, and changes made via the state setters in context.
 */
const UserProvider: React.FC = ({ children }) => {
  const puzzleSolutionsStorageHandler = useStaticValue<
    StorageHandler<PuzzleSolutions>
  >(() => new StorageHandler('puzzleSolutions'));
  const [puzzleSolutions, setPuzzleSolutions] = useState<PuzzleSolutions>(
    puzzleSolutionsStorageHandler.get()?.value ?? defaultSolutions
  );

  // Persist solutions in local storage
  useEffect(() => {
    puzzleSolutionsStorageHandler.set(puzzleSolutions);
  }, [puzzleSolutionsStorageHandler, puzzleSolutions]);

  // Helpers for a checks that we use in each setter
  const assertSolutionExists = (puzzleSlug: string, fileName: string): void => {
    const solutionsForPuzzle = puzzleSolutions[puzzleSlug] ?? {};
    if (!(fileName in solutionsForPuzzle)) {
      throw new Error(`Solution ${fileName} does not exist for this puzzle`);
    }
  };
  const assertSolutionNotExists = (
    puzzleSlug: string,
    fileName: string
  ): void => {
    const solutionsForPuzzle = puzzleSolutions[puzzleSlug] ?? {};
    if (fileName in solutionsForPuzzle) {
      throw new Error(`Solution ${fileName} already exists for this puzzle`);
    }
  };

  const getPuzzleSolutions = (puzzleSlug: string): PuzzleSolution[] => {
    return Object.values(puzzleSolutions[puzzleSlug] ?? {});
  };
  const getPuzzleSolution = (
    puzzleSlug: string,
    fileName: string
  ): PuzzleSolution | undefined => {
    return puzzleSolutions[puzzleSlug]?.[fileName];
  };
  const createSolution = (puzzleSlug: string, fileName: string): void => {
    assertSolutionNotExists(puzzleSlug, fileName);
    setPuzzleSolutions((old) => ({
      ...old,
      [puzzleSlug]: {
        ...old[puzzleSlug],
        [fileName]: {
          fileName,
          sourceCode: '',
        },
      },
    }));
  };
  const copySolution = (puzzleSlug: string, fileName: string): void => {
    assertSolutionExists(puzzleSlug, fileName);

    const solutionsForPuzzle = puzzleSolutions[puzzleSlug] ?? {};
    const existingSolution = solutionsForPuzzle[fileName];
    // Keep tacking on 'copy' to the name until we get an unused file name
    let newFileName = fileName;
    while (newFileName in solutionsForPuzzle) {
      newFileName += ' copy';
    }
    setPuzzleSolutions((old) => ({
      ...old,
      [puzzleSlug]: {
        ...old[puzzleSlug],
        [newFileName]: { ...existingSolution, fileName: newFileName },
      },
    }));
  };
  const renameSolution = (
    puzzleSlug: string,
    oldFileName: string,
    newFileName: string
  ): void => {
    assertSolutionExists(puzzleSlug, oldFileName);
    assertSolutionNotExists(puzzleSlug, newFileName);

    const existingSolution = puzzleSolutions[puzzleSlug][oldFileName];
    setPuzzleSolutions((old) => ({
      ...old,
      [puzzleSlug]: {
        ...omit(old[puzzleSlug], oldFileName),
        [newFileName]: { ...existingSolution, fileName: newFileName },
      },
    }));
  };
  const deleteSolution = (puzzleSlug: string, fileName: string): void => {
    assertSolutionExists(puzzleSlug, fileName);

    setPuzzleSolutions((old) => ({
      ...old,
      [puzzleSlug]: {
        ...omit(old[puzzleSlug], fileName),
      },
    }));
  };
  const setSolutionSourceCode = (
    puzzleSlug: string,
    fileName: string,
    sourceCode: string
  ): void => {
    assertSolutionExists(puzzleSlug, fileName);

    const existingSolution = puzzleSolutions[puzzleSlug][fileName];
    setPuzzleSolutions((old) => ({
      ...old,
      [puzzleSlug]: {
        ...old[puzzleSlug],
        [fileName]: {
          ...existingSolution,
          sourceCode,
        },
      },
    }));
  };

  return (
    <PuzzleSolutionsContext.Provider
      value={{
        getPuzzleSolutions,
        getPuzzleSolution,
        createSolution,
        copySolution,
        renameSolution,
        deleteSolution,
        setSolutionSourceCode,
      }}
    >
      {children}
    </PuzzleSolutionsContext.Provider>
  );
};

export default UserProvider;
