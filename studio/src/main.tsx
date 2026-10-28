import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App, MissingToken } from "./App";
import { takeToken } from "./api";
import "./styles.css";

const root = document.getElementById("root");
if (root) {
  const token = takeToken();
  createRoot(root).render(
    <StrictMode>{token ? <App token={token} /> : <MissingToken />}</StrictMode>,
  );
}
