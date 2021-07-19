import { CssBaseline, CircularProgress } from '@material-ui/core';
import React, { Suspense } from 'react';
import { ThemeProvider } from '@material-ui/styles';
import theme from '@root/util/theme';
import CoreContent from './CoreContent';
import { BrowserRouter } from 'react-router-dom';

/**
 * Root component in the app
 */
const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Suspense fallback={<CircularProgress />}>
        <BrowserRouter>
          <CoreContent />
        </BrowserRouter>
      </Suspense>
    </ThemeProvider>
  );
};

export default App;
