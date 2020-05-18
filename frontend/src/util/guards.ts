/**
 * Asserts that the given value is not `null` or `undefined`.
 * @param val The value to check
 * @throws Error if the value is `null` or `undefined`
 */
export function assertIsDefined<T>(val: T): asserts val is NonNullable<T> {
  if (val === undefined || val === null) {
    throw new Error(`Expected 'val' to be defined, but received ${val}`);
  }
}

/**
 * Type guard for any uniformly typed array. Checks that the input is an array,
 * and that each element is of the specified type.
 *
 * @param elementGuard A type guard function for the individual element type.
 *  Will be called on each element (if the input is an array)
 * @param value The value to check
 * @returns true if the object is a T[], false if not
 */
export function isTypedArray<T>(
  elementGuard: (e: T) => e is T,
  value: unknown
): value is T[] {
  return value instanceof Array && value.every((e) => elementGuard(e));
}
