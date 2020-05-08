import { Theme, createMuiTheme } from '@material-ui/core';
import { blue, red } from '@material-ui/core/colors';

const theme: Theme = createMuiTheme({
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
    fontFamily: 'monospace',
    fontWeightLight: 400,
    fontWeightRegular: 600,
    fontWeightMedium: 600,
    fontWeightBold: 900,
  },

  props: {
    MuiSnackbar: {
      autoHideDuration: 3000,
      anchorOrigin: { vertical: 'bottom', horizontal: 'left' },
    },
  },
});

export default theme;
