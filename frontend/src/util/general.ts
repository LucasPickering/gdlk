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

export function sum<T = number>(
  values: T[],
  mapper: (value: T) => number
): number {
  return values.reduce((acc, value) => acc + mapper(value), 0);
}
