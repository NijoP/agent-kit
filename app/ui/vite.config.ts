import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The UI is a pure web app during development: it plays a captured `eak-events.jsonl`
// (real kernel output) through the same fold the packaged Tauri app will drive live.
// No webkit2gtk / Tauri toolchain is needed to run `npm run dev` — that is only required
// when the app is finally packaged as a desktop binary. See README.md.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: false },
  test: {
    globals: true,
    environment: "node",
  },
});
