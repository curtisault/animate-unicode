import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import elm from "vite-plugin-elm";

// The shell is imported from its TypeScript source, so the dev loop needs only
// `mise run build:wasm` (which writes web/pkg), not a build of web/dist.
const web = (p: string) => fileURLToPath(new URL(`../web/src/${p}`, import.meta.url));

export default defineConfig({
  plugins: [elm()],
  resolve: {
    alias: [
      { find: "animate-unicode/element", replacement: web("element.ts") },
      { find: "animate-unicode", replacement: web("index.ts") },
    ],
  },
  server: {
    // The shell and the wasm live one folder up, beside the site.
    fs: { allow: [".."] },
  },
  build: {
    target: "es2022",
    // test.html is the shell's checklist page (Phase 3), plain HTML beside the Elm site.
    rollupOptions: {
      input: { main: fileURLToPath(new URL("index.html", import.meta.url)), test: fileURLToPath(new URL("test.html", import.meta.url)) },
    },
  },
});
