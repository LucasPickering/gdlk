import { useRef } from 'react';

/**
 * Store a static, immutable value that is initialized lazily. An alternative
 * to useRef, for when the value is expensive to initialize but doesn't need to
 * be changed.
 * @param init The function to initialize the static value
 * @return The initialized value.
 */
const useStaticValue = <T>(init: () => T): T => {
  const ref = useRef<{ initialized: boolean; value: T | undefined }>({
    initialized: false,
    value: undefined,
  });

  if (!ref.current.initialized) {
    ref.current = { initialized: true, value: init() };
  }

  // At this point we know `value` is a T, but we  have to convince the typechecker
  return ref.current.value as T;
};

export default useStaticValue;
