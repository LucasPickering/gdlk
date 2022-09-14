import { makeStyles } from "@mui/styles";
import React from "react";
import clsx from "clsx";
import HeaderBar from "../header/HeaderBar";

const useLocalStyles = makeStyles(({ palette, spacing }) => ({
  pageContainer: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    height: "100%",
  },
  pageBody: {
    width: "100%",
  },
  pageBodyNotFullScreen: {
    maxWidth: 1280,
    padding: spacing(2),
    paddingBottom: 0,
  },
  pageBodyFullScreen: {
    height: "100%",
    overflowY: "hidden",
  },
  pageFooter: {
    marginTop: "auto",
    padding: spacing(2),
    display: "flex",
    justifyContent: "center",
    "& > *": {
      padding: `0px ${spacing(0.5)}`,
    },
    "& > * + *": {
      borderLeftWidth: 1,
      borderLeftStyle: "solid",
      borderLeftColor: palette.divider,
    },
  },
}));

interface Props {
  fullScreen: boolean;
  children?: React.ReactNode;
}

/**
 * Container for all content on the page. This is used in the root to wrap all
 * pages.
 */
const PageContainer: React.FC<Props> & { defaultProps: Partial<Props> } = ({
  fullScreen,
  children,
}) => {
  const localClasses = useLocalStyles();

  return (
    <div className={localClasses.pageContainer}>
      {!fullScreen && <HeaderBar />}

      <div
        className={clsx(
          localClasses.pageBody,
          fullScreen
            ? localClasses.pageBodyFullScreen
            : localClasses.pageBodyNotFullScreen
        )}
      >
        {children}
      </div>
    </div>
  );
};

PageContainer.defaultProps = {
  fullScreen: false,
};

export default PageContainer;
