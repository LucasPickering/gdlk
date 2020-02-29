import { Theme, createMuiTheme } from '@material-ui/core';
import { deepOrange, red } from '@material-ui/core/colors';
import { ThemeOptions } from '@material-ui/core/styles/createMuiTheme';

const createCustomTheme = (options: ThemeOptions): Theme =>
  createMuiTheme(options);

const theme: Theme = createCustomTheme({
  palette: {
    type: 'dark',
    primary: deepOrange,
    secondary: red,
  },
});

export default theme;
