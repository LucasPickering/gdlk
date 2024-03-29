import React, { Suspense } from "react";
import HeaderBar from "../header/HeaderBar";
import { useMatch } from "react-router-dom";
import { Box, CircularProgress } from "@mui/material";

interface Props {
  children?: React.ReactNode;
}

/**
 * Container for all content on the page. This is used in the root to wrap all
 * pages.
 */
const PageContainer: React.FC<Props> = ({ children }) => {
  const fullScreen = Boolean(useMatch("/puzzles/:puzzleSlug/solution"));

  return (
    <Box
      display="flex"
      flexDirection="column"
      alignItems="center"
      height="100%"
    >
      <HeaderBar />

      <Box
        width="100%"
        sx={
          fullScreen
            ? {
                height: "100%",
                overflowY: "hidden",
              }
            : ({ spacing }) => ({
                maxWidth: 1400,
                padding: spacing(2),
                paddingBottom: 0,
              })
        }
      >
        {/* Scope laoding state to the page content. Useful for IDE loading */}
        <Suspense fallback={<CircularProgress />}>{children}</Suspense>
      </Box>
    </Box>
  );
};

export default PageContainer;
