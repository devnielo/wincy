import { getCurrentWindow } from "@tauri-apps/api/window";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { isTauri } from "./api";
import { PreviewWindow } from "./components/PreviewWindow";
import "./index.css";

const isPreview = isTauri && getCurrentWindow().label === "preview";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{isPreview ? <PreviewWindow /> : <App />}</React.StrictMode>,
);

if (!isTauri) {
  // Outside Tauri there is no native Mica behind the page: index.css paints a stand-in backdrop.
  document.documentElement.dataset.browser = "";
  // ...and no second window: the preview panel renders beside the palette instead.
  const preview = document.createElement("div");
  preview.id = "preview-root";
  document.body.append(preview);
  ReactDOM.createRoot(preview).render(<PreviewWindow />);
}
