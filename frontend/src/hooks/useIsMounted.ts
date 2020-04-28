import { useRef, useEffect } from 'react';

const useIsMounted = (): React.MutableRefObject<boolean> => {
  const mounted = useRef(true);
  useEffect(
    // Return an unmount callback that sets the flag
    () => () => {
      mounted.current = false;
    },
    [mounted]
  );
  return mounted;
};

export default useIsMounted;
