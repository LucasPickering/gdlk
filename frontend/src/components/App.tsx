import { CssBaseline, CircularProgress } from '@material-ui/core';
import React, { Suspense } from 'react';
import { ThemeProvider } from '@material-ui/styles';
import theme from 'util/theme';
import environment from 'util/environment';
import { RelayEnvironmentProvider } from 'relay-hooks';
import CoreContent from './CoreContent';
import { BrowserRouter } from 'react-router-dom';

/**
 * Root component in the app
 */
const App: React.FC = () => {
  return (
    <RelayEnvironmentProvider environment={environment}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <Suspense fallback={<CircularProgress />}>
          <BrowserRouter>
            <CoreContent />
          </BrowserRouter>
        </Suspense>
      </ThemeProvider>
    </RelayEnvironmentProvider>
  );
};

export default App;
