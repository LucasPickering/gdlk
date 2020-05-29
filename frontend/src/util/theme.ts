import { Theme, createMuiTheme, responsiveFontSizes } from '@material-ui/core';
import { blue, red } from '@material-ui/core/colors';

const theme: Theme = responsiveFontSizes(
  createMuiTheme({
    palette: {
      type: 'dark',
      primary: blue,
      secondary: red, // for error contexts ONLY
      divider: '#ffffff',
      background: {
        default: '#000000',
        paper: '#202020',
      },
    },
    typography: {
      // Makes math for `rem` font sizes easy
      // https://www.sitepoint.com/understanding-and-using-rem-units-in-css/
      htmlFontSize: 10,
      fontFamily: 'Consolas, monospace',
      fontWeightLight: 400,
      fontWeightRegular: 600,
      fontWeightMedium: 600,
      fontWeightBold: 900,

      h1: {
        fontSize: '3.2rem',
      },
      h2: {
        fontSize: '2.8rem',
      },
      h3: {
        fontSize: '2.4rem',
      },
      h4: {
        fontSize: '2.0rem',
      },
      h5: {
        fontSize: '1.6rem',
      },
      h6: {
        fontSize: '1.2rem',
      },
    },

    props: {
      MuiCard: {
        component: 'section',
      },
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
        anchorOrigin: { vertical: 'bottom', horizontal: 'left' },
      },
    },
  })
);

export default theme;
