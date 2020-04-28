import { CssBaseline } from '@material-ui/core';
import React from 'react';
import { ThemeProvider } from '@material-ui/styles';
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import theme from 'util/theme';
import HomePage from './HomePage';
import HardwareSpecView from './hardware/HardwareSpecView';
import ProgramSpecView from './programs/ProgramSpecView';
import NotFoundPage from './NotFoundPage';
import PageContainer from './common/PageContainer';
import ProgramIdeView from './ide/ProgramIdeView';

const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <BrowserRouter>
        <Switch>
          {/* Full screen routes first */}
          <Route path="/hardware/:hwSlug/programs/:programSlug/:fileName" exact>
            <PageContainer fullScreen>
              <ProgramIdeView />
            </PageContainer>
          </Route>

          {/* All non-full screen routes */}
          <Route path="*">
            <PageContainer>
              <Switch>
                <Route path="/" exact>
                  <HomePage />
                </Route>

                {/* Hardware routes */}
                <Route path="/hardware/:hwSlug" exact>
                  <HardwareSpecView />
                </Route>
                <Route path="/hardware/:hwSlug/programs/:programSlug" exact>
                  <ProgramSpecView />
                </Route>

                <Route path="*">
                  <NotFoundPage />
                </Route>
              </Switch>
            </PageContainer>
          </Route>
        </Switch>
      </BrowserRouter>
    </ThemeProvider>
  );
};

export default App;
