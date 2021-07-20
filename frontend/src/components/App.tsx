import { CssBaseline, CircularProgress } from '@material-ui/core';
import React, { Suspense } from 'react';
import { ThemeProvider } from '@material-ui/styles';
import theme from '@root/util/theme';
import CoreContent from './CoreContent';
import { HashRouter } from 'react-router-dom';

/**
 * Root component in the app
 */
const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Suspense fallback={<CircularProgress />}>
        <HashRouter>
          <CoreContent />
        </HashRouter>
      </Suspense>
    </ThemeProvider>
  );
};

export default App;
