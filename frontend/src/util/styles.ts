import { Breakpoints } from "@mui/material";

export type ScreenSize = "small" | "large";

/**
 * Creates a media query for the specified screen size. To be used in makeStyles
 */
export const sizeMq = (size: ScreenSize, breakpoints: Breakpoints): string =>
  size === "large" ? breakpoints.up("sm") : breakpoints.only("xs");
