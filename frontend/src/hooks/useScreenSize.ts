import { ScreenSize } from 'util/styles';
import { useMediaQuery, useTheme } from '@material-ui/core';

/**
 * Gets the current screen size.
 */
const useScreenSize = (): ScreenSize => {
  // If you update the logic here, make sure to change it in styles.ts too
  const { breakpoints } = useTheme();
  return useMediaQuery(breakpoints.up('sm')) ? 'large' : 'small';
};

export default useScreenSize;
