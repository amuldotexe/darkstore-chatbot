import "./styles.css";

import { createConciergeApplication } from "./app";
import { createTauriConciergeBridge } from "./bridge";

const root = document.querySelector<HTMLElement>("#app");
if (!root) {
  throw new Error("Missing application root.");
}

const application = createConciergeApplication(root, createTauriConciergeBridge());
application.mountApplicationScreen();
