import { Puzzle } from '@root/util/types';

export const puzzles: Record<string, Puzzle> = {
  // TODO de-dupe key+slug
  readWrite: {
    name: 'Read/Write',
    slug: 'readWrite',
    description: 'Read a value from input and write it to output.',
    examples: [],
    input: [1],
    expectedOutput: [1],
  },
  readWrite3: {
    name: 'Read 3/Write 3',
    slug: 'readWrite3',
    description: 'Read three values from input and write them to output.',
    examples: [],
    input: [1, 1, 1],
    expectedOutput: [1, 1, 1],
  },
  inAndOut: {
    name: 'In-N-Out',
    slug: 'inAndOut',
    description:
      'Read a sequence of values from input and write them all to output.',
    examples: [{ input: [1, 2, 3], output: [1, 2, 3] }],
    input: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    expectedOutput: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
  },
  oddEven: {
    name: 'Odd/Even',
    slug: 'oddEven',
    description:
      'Given an input of positive integers, determine if each one is even or odd. If even, output 0, if odd, output 1.',
    examples: [{ input: [1, 2, 3, 4], output: [1, 0, 1, 0] }],
    input: [
      43, 48, 62, 70, 69, 91, 78, 46, 72, 21, 67, 49, 49, 5, 3, 18, 26, 52, 94,
      18, 63, 62, 100, 33, 27, 99, 38, 58, 40, 3, 83, 24, 26, 14, 17, 78, 11,
      28, 84, 93,
    ],
    expectedOutput: [
      1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1,
      1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 1,
    ],
  },
  divisible: {
    name: 'Divisible',
    slug: 'divisible',
    description:
      'Given an input of `(a, b)` pairs, determine if `b` is divisible by `a`. Output 0 if not divisible, 1 if divisible.',
    examples: [{ input: [3, 12, 3, 10, 5, 10], output: [1, 0, 1] }],
    input: [
      5, 68, 5, 91, 7, 78, 6, 15, 4, 2, 7, 100, 7, 75, 4, 38, 7, 84, 5, 47, 7,
      45, 7, 99, 4, 33, 5, 75, 7, 36, 3, 20, 5, 25, 5, 21, 5, 22, 5, 30,
    ],
    expectedOutput: [
      0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1,
    ],
  },
  sort: {
    name: 'Sort',
    slug: 'sort',
    description: 'Sort the input list in ascending order.',
    examples: [{ input: [5, 3, 4, 1, 2], output: [1, 2, 3, 4, 5] }],
    input: [9, 3, 8, 4, 5, 1, 3, 8, 9, 5, 2, 10, 4, 1, 8],
    expectedOutput: [1, 1, 2, 3, 3, 4, 4, 5, 5, 8, 8, 8, 9, 9, 10],
  },
};
