import React from "react";
import "./App.css";
import { Terminal } from "./terminal/Terminal";
const App: React.FC = () => {
  return (
    <div className="App">
      <header className="App-header">
        <p>poopie</p>
        <Terminal />
      </header>
    </div>
  );
};

export default App;
