import React from "react";
import { Routes, Route, Outlet } from "react-router-dom";
import HomePage from "./HomePage";
import NotFoundPage from "./NotFoundPage";
import PageContainer from "./common/PageContainer";
import DocsPage from "@root/components/docs/DocsPage";
import AboutPage from "./AboutPage";
import HardwareCard from "./hardware/HardwareCard";
import PuzzleListView from "./puzzle/PuzzleListView";

const ProgramIdeView = React.lazy(() => import("./ide/ProgramIdeView"));

/**
 * Child of the root component. Expects Material UI to be set up in
 * the parent. Also handles loading of global API content.
 */
const CoreContent: React.FC = () => {
  return (
    <Routes>
      <Route
        element={
          <PageContainer>
            <Outlet />
          </PageContainer>
        }
      >
        {/* Most routes get rendered within the home nav page */}
        <Route path="/" element={<HomePage />}>
          <Route path="/puzzles" element={<PuzzleListView />} />
          <Route path="/hardware" element={<HardwareCard />} />
          <Route path="/docs" element={<DocsPage />} />
          <Route path="/about" element={<AboutPage />} />
        </Route>

        {/* Fullscreen routes, which aren't rendered within the home page */}
        <Route
          path="/puzzles/:puzzleSlug/solution"
          element={<ProgramIdeView />}
        />

        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
};

export default CoreContent;
