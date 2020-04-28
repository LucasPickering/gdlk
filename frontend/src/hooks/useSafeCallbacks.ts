import { useRef } from 'react';
import { isEqual } from 'lodash';

/**
 * Hook that provides a safety check on an object of callbacks, making sure
 * none of them changed. We want to prevent bugs where sockets get re-opened
 * or requests get kicked off repeatedly. This will typically force the caller
 * to wrap their callbacks in useCallback.
 */
const useSafeCallbacks = <T extends object>(callbacks: T): T => {
  // Safety check to make sure callbacks don't change. We expect the outer
  // object to change, but not the callbacks themselves. This check prevents
  // unintended triggers on the useEffect hook.
  const callbacksRef = useRef<object>(callbacks);
  if (!isEqual(callbacks, callbacksRef.current)) {
    throw new Error('The callbacks object changed. You broke the rules!');
  }
  return callbacks;
};

export default useSafeCallbacks;
