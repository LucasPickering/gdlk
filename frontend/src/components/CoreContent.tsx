import React from 'react';
import { Switch, Route } from 'react-router-dom';
import graphql from 'babel-plugin-relay/macro';
import HomePage from './HomePage';
import HardwareSpecDetailsView from './hardware/HardwareSpecDetailsView';
import ProgramSpecView from './programs/ProgramSpecView';
import NotFoundPage from './NotFoundPage';
import PageContainer from './common/PageContainer';
import DocsPage from 'components/docs/DocsPage';
import { UserContext, UserContextType, defaultUserContext } from 'state/user';
import { useQuery } from 'relay-hooks';
import { CoreContentQuery } from './__generated__/CoreContentQuery.graphql';
import InitializeUserDialog from './user/InitializeUserDialog';

const ProgramIdeView = React.lazy(() => import('./ide/ProgramIdeView'));

const ROOT_QUERY = graphql`
  query CoreContentQuery {
    authStatus {
      authenticated
      userCreated
      user {
        id
        username
      }
    }
  }
`;

/**
 * Child of the root component. Expects Relay and Material UI to be set up in
 * the parent. Also handles loading of global API content.
 */
const CoreContent: React.FC = () => {
  // Query for global data, like auth status
  const { props, retry } = useQuery<CoreContentQuery>(ROOT_QUERY);
  const userContext: UserContextType = props
    ? {
        loggedInUnsafe: props.authStatus.authenticated,
        loggedIn: props.authStatus.userCreated,
        user: props.authStatus.user ?? undefined,
        refetch: retry,
      }
    : defaultUserContext;

  return (
    <UserContext.Provider value={userContext}>
      {/* If the user is logged in but hasn't finished setup, show the setup
          modal */}
      {userContext.loggedInUnsafe && !userContext.loggedIn && (
        <InitializeUserDialog />
      )}

      <Switch>
        {/* Full screen routes first */}
        <Route path="/hardware/:hwSlug/puzzles/:programSlug/:fileName" exact>
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

              {/* Hardware routes */}
              <Route path="/hardware/:hwSlug" exact>
                <HardwareSpecDetailsView />
              </Route>
              <Route path="/hardware/:hwSlug/puzzles/:programSlug" exact>
                <ProgramSpecView />
              </Route>

              <Route path="*">
                <NotFoundPage />
              </Route>
            </Switch>
          </PageContainer>
        </Route>
      </Switch>
    </UserContext.Provider>
  );
};

export default CoreContent;
