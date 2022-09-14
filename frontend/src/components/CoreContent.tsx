import React from "react";
import { Routes, Route, Outlet } from "react-router-dom";
import HomePage from "./HomePage";
import PuzzleDetailsView from "./puzzle/PuzzleDetailsView";
import NotFoundPage from "./NotFoundPage";
import PageContainer from "./common/PageContainer";
import DocsPage from "@root/components/docs/DocsPage";
import AboutPage from "./AboutPage";
import HardwareCard from "./hardware/HardwareCard";

const ProgramIdeView = React.lazy(() => import("./ide/ProgramIdeView"));

/**
 * Child of the root component. Expects Material UI to be set up in
 * the parent. Also handles loading of global API content.
 */
const CoreContent: React.FC = () => {
  return (
    <Routes>
      {/* Full screen routes first */}
      <Route
        path="/puzzles/:puzzleSlug/solution"
        element={<ProgramIdeView />}
      />

      {/* All non-full screen routes */}
      <Route
        element={
          <PageContainer>
            <Outlet />
          </PageContainer>
        }
      >
        <Route path="/" element={<HomePage />}>
          <Route path="/puzzles" />
          <Route path="/hardware" element={<HardwareCard />} />
          <Route path="/puzzles/:puzzleSlug" element={<PuzzleDetailsView />} />
          <Route path="/docs" element={<DocsPage />} />
          <Route path="/about" element={<AboutPage />} />
        </Route>

        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
};

export default CoreContent;
