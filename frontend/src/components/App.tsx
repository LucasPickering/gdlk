import { CssBaseline } from '@material-ui/core';
import React from 'react';
import { ThemeProvider } from '@material-ui/styles';
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import Terminal from './Terminal';
import theme from 'util/theme';
import HomeView from './HomeView';
import HardwareSpecView from './hardwareSpec/HardwareSpecView';
import NotFound from './NotFound';

const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <BrowserRouter>
        <Switch>
          <Route path="/terminal" exact>
            <Terminal />
          </Route>
          <Route path="/" exact>
            <HomeView />
          </Route>
          <Route path="/hardware/:hwSlug">
            <HardwareSpecView />
          </Route>
          <Route path="*">
            <NotFound />
          </Route>
        </Switch>
      </BrowserRouter>
    </ThemeProvider>
  );
};

export default App;
