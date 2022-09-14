import { Theme, responsiveFontSizes } from "@mui/material";
import {
  createTheme,
  DeprecatedThemeOptions,
  adaptV4Theme,
} from "@mui/material/styles";

const theme: Theme = (() => {
  // We have to create theme theme twice:
  // - First time, with the basic global options
  // - Second time including component-specific overrides
  // This allows us to reference the base theme in the overrides
  const config: DeprecatedThemeOptions = {
    palette: {
      // These colors are supposed to mimic the ANSI base 8
      mode: "dark",
      primary: {
        // ANSI cyan
        main: "#5ac2c6",
      },
      error: {
        main: "#ff0000",
      },
      divider: "#ffffff",
      action: {
        hover: "#ffffff",
        selected: "#ffffff",
      },
      success: {
        main: "#00ff00",
      },
      background: {
        default: "#000000",
        paper: "#202020",
      },
    },

    typography: {
      // Makes math for `rem` font sizes easy
      // https://www.sitepoint.com/understanding-and-using-rem-units-in-css/
      htmlFontSize: 10,
      fontFamily: "Consolas, monospace",
      fontWeightLight: 400,
      fontWeightRegular: 600,
      fontWeightMedium: 600,
      fontWeightBold: 900,

      h1: {
        fontSize: "3.2rem",
      },
      h2: {
        fontSize: "2.8rem",
      },
      h3: {
        fontSize: "2.4rem",
      },
      h4: {
        fontSize: "2.0rem",
      },
      h5: {
        fontSize: "1.6rem",
      },
      h6: {
        fontSize: "1.2rem",
      },
    },

    props: {
      MuiCardHeader: {
        // CardHeader enforces that the component is always 'span' which is shit
        // so we just supply our own Typography everywhere
        disableTypography: false,
      },
      MuiGrid: {
        spacing: 2,
      },
      MuiSnackbar: {
        autoHideDuration: 3000,
        anchorOrigin: { vertical: "bottom", horizontal: "left" },
      },
      MuiTypography: {
        // <p> tags aren't actually what we want in most cases, so make them
        // explicit.
        // NOTE: do *NOT* override `component` here, or it will effectively
        // disable the `paragraph` and `variantMapping` props in all places,
        // since `component` always takes precedence
        variantMapping: {
          body1: "div",
          body2: "div",
          inherit: "div",
        },
      },
    },
  };

  const theme = createTheme(adaptV4Theme(config));

  // Create the real deal now
  return responsiveFontSizes(
    createTheme(
      adaptV4Theme({
        ...config,
        overrides: {
          MuiButton: {
            root: {
              borderRadius: 0,
            },
            contained: {
              backgroundColor: theme.palette.common.white,
            },
          },
          MuiListItem: {
            button: {
              "&:hover": {
                color: theme.palette.getContrastText(
                  theme.palette.action.hover
                ),
              },
              // We _should_ be able to do this as another override for the
              // component, but that class never got applied so I had to this
              // as a workaround
              "&.Mui-selected": {
                color: theme.palette.getContrastText(
                  theme.palette.action.selected
                ),
                "&::before": {
                  content: '">"',
                  paddingRight: 8,
                },
              },
            },
          },
        },
      })
    )
  );
})();

export default theme;
