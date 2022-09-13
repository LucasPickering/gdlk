/**
 * Check if a value is not null/undefined
 */
export function isDefined<T>(value: T): value is NonNullable<T> {
  return value !== null && value !== undefined;
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
