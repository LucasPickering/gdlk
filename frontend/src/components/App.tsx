import React from "react";
import "./App.css";
import Terminal from "./Terminal";
const App: React.FC = () => {
  return (
    <div className="App">
      <header className="App-header">
        <Terminal />
      </header>
    </div>
  );
};

export default App;
