import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App, MissingToken } from "./App";
import { takeToken } from "./api";
import "./styles.css";
import "./workspace/theme.css";
import "./workspace/workspace.css";
import { Boundary } from "./workspace/Boundary";

const root = document.getElementById("root");
if (root) {
  const token = takeToken();
  createRoot(root).render(
    <StrictMode>
      <Boundary name="Studio">
        {token ? <App token={token} /> : <MissingToken />}
      </Boundary>
    </StrictMode>,
  );
}
