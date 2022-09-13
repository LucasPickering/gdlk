import React from "react";
import { Switch, Route } from "react-router-dom";
import HomePage from "./HomePage";
import PuzzleDetailsView from "./puzzle/PuzzleDetailsView";
import NotFoundPage from "./NotFoundPage";
import PageContainer from "./common/PageContainer";
import DocsPage from "@root/components/docs/DocsPage";

const ProgramIdeView = React.lazy(() => import("./ide/ProgramIdeView"));

/**
 * Child of the root component. Expects Material UI to be set up in
 * the parent. Also handles loading of global API content.
 */
const CoreContent: React.FC = () => {
  return (
    <Switch>
      {/* Full screen routes first */}
      <Route path="/puzzles/:puzzleSlug/solution" exact>
        <ProgramIdeView />
      </Route>

      {/* All non-full screen routes */}
      <Route path="*">
        <PageContainer>
          <Switch>
            <Route path={["/", "/hardware", "/puzzles/:puzzleSlug?"]} exact>
              <HomePage />
            </Route>

            <Route path="/docs">
              <DocsPage />
            </Route>

            <Route path="/puzzles/:puzzleSlug" exact>
              <PuzzleDetailsView />
            </Route>

            <Route path="*">
              <NotFoundPage />
            </Route>
          </Switch>
        </PageContainer>
      </Route>
    </Switch>
  );
};

export default CoreContent;
