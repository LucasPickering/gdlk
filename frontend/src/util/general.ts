/**
 * Map the values of an object, keeping the corresponding keys intact.
 */
export function mapValues<T extends Record<string, unknown>, V>(
  obj: T,
  mapper: (value: T[keyof T], key: keyof T) => V
): { [key in keyof T]: V } {
  return Object.entries(obj).reduce((acc, [key, value]) => {
    // Object.entries is shit and strips the typing, so we need to coerce the
    // key and value back to the original type here
    acc[key as keyof T] = mapper(value as T[keyof T], key);
    return acc;
  }, {} as { [key in keyof T]: V });
}

/**
 * Sum an array of values by mapping each one to a number.
 */
export function sum<T = number>(
  values: T[],
  mapper: (value: T) => number
): number {
  return values.reduce((acc, value) => acc + mapper(value), 0);
}

/**
 * Generate an array of sequential numbers with a given start and end.
 * @param start First value in the array, inclusive
 * @param end End of the array, exclusive
 * @returns An array of numbers, of length `end-start`
 */
export function range(start: number, end: number): number[] {
  const array = Array(end - start);
  for (let i = start; i < end; i++) {
    array.push(i);
  }
  return array;
}
