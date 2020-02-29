import React from 'react';
import { BrowserRouter, Switch, Route } from 'react-router-dom';
import Terminal from './Terminal';

const App: React.FC = () => {
  return (
    <BrowserRouter>
      <Switch>
        <Route path="/terminal">
          <Terminal />
        </Route>
      </Switch>
    </BrowserRouter>
  );
};

export default App;
