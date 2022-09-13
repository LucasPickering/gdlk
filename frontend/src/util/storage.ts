export interface StorageValue<T> {
  value: T;
  lastModified: Date;
}

/**
 * A utility class to handle storing values in browser local storage.This
 * handles serializing/deserialize the value via JSON, and also tags it with
 * a lastModified field automatically.
 */
export class StorageHandler<T> {
  private key: string;

  constructor(key: string) {
    this.key = key;
  }

  public get(): StorageValue<T> | undefined {
    const rawValue = window.localStorage.getItem(this.key);
    if (rawValue !== null) {
      try {
        const objValue = JSON.parse(rawValue);

        // If we get an unexpected value, just return undefined
        if (
          typeof objValue === "object" &&
          typeof objValue.value !== undefined &&
          typeof objValue.lastModified === "string"
        ) {
          return {
            value: objValue.value as T,
            lastModified: new Date(objValue.lastModified),
          };
        }
      } catch (e) {
        // Invalid value, just fall through to the default case
      }
    }

    return undefined;
  }

  public set(value: T): void {
    const storedObj: StorageValue<T> = {
      value,
      lastModified: new Date(), // This will be serialized as an ISO string
    };
    window.localStorage.setItem(this.key, JSON.stringify(storedObj));
  }

  public delete(): void {
    window.localStorage.removeItem(this.key);
  }
}
