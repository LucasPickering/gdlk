import { Theme, createMuiTheme } from '@material-ui/core';
import { green, red } from '@material-ui/core/colors';

const theme: Theme = createMuiTheme({
  palette: {
    type: 'dark',
    primary: green,
    secondary: red, // for error contexts ONLY
    divider: '#ffffff',
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
});

export default theme;
