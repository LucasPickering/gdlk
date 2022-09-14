import {
  CssBaseline,
  CircularProgress,
  StyledEngineProvider,
  Theme,
  ThemeProvider,
} from "@mui/material";
import React, { Suspense } from "react";
import theme from "@root/util/theme";
import CoreContent from "./CoreContent";
import { BrowserRouter } from "react-router-dom";
import { RecoilRoot } from "recoil";

declare module "@mui/styles/defaultTheme" {
  // eslint-disable-next-line @typescript-eslint/no-empty-interface
  interface DefaultTheme extends Theme {}
}

/**
 * Root component in the app
 */
const App: React.FC = () => {
  return (
    <StyledEngineProvider injectFirst>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <Suspense fallback={<CircularProgress />}>
          <RecoilRoot>
            <BrowserRouter>
              <CoreContent />
            </BrowserRouter>
          </RecoilRoot>
        </Suspense>
      </ThemeProvider>
    </StyledEngineProvider>
  );
};

export default App;
