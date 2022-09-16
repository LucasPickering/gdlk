/**
 * Check if a value is not null/undefined
 */
export function isDefined<T>(value: T): value is NonNullable<T> {
  return value !== null && value !== undefined;
}

/**
 * Assert the given value is defined. Useful as a type guard when you know
 * something is defined but the typechecker doesn't.
 */
export function assertIsDefined<T>(
  value: T,
  message: string = ""
): asserts value is NonNullable<T> {
  if (!isDefined(value)) {
    throw new Error(
      `Expected value to be defined, but was ${value}. ${message}`
    );
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
