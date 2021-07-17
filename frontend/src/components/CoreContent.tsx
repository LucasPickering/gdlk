import React from 'react';
import { Switch, Route } from 'react-router-dom';
import HomePage from './HomePage';
import HardwareSpecDetailsView from './hardware/HardwareSpecDetailsView';
import PuzzleView from './puzzle/PuzzleView';
import NotFoundPage from './NotFoundPage';
import PageContainer from './common/PageContainer';
import DocsPage from '@root/components/docs/DocsPage';
import UserProvider from './UserProvider';

const ProgramIdeView = React.lazy(() => import('./ide/ProgramIdeView'));

/**
 * Child of the root component. Expects Material UI to be set up in
 * the parent. Also handles loading of global API content.
 */
const CoreContent: React.FC = () => {
  return (
    <UserProvider>
      <Switch>
        {/* Full screen routes first */}
        <Route path="/puzzles/:puzzleSlug/:fileName" exact>
          <ProgramIdeView />
        </Route>

        {/* All non-full screen routes */}
        <Route path="*">
          <PageContainer>
            <Switch>
              <Route path="/" exact>
                <HomePage />
              </Route>

              <Route path="/docs">
                <DocsPage />
              </Route>

              <Route path="/hardware/:hwSlug" exact>
                <HardwareSpecDetailsView />
              </Route>
              <Route path="/puzzles/:puzzleSlug" exact>
                <PuzzleView />
              </Route>

              <Route path="*">
                <NotFoundPage />
              </Route>
            </Switch>
          </PageContainer>
        </Route>
      </Switch>
    </UserProvider>
  );
};

export default CoreContent;
