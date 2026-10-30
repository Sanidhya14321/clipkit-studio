// Bundles src/Root.tsx (and the studio composition it imports) once into ./bundle, the serve
// URL every render uses. React and Remotion resolve to this package's copies so the bundle
// never holds two Reacts.
import { bundle } from "@remotion/bundler";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const pkg = (name) => dirname(require.resolve(`${name}/package.json`));

const out = await bundle({
  entryPoint: resolve(here, "Root.tsx"),
  outDir: resolve(here, "..", "bundle"),
  webpackOverride: (config) => ({
    ...config,
    resolve: {
      ...config.resolve,
      alias: {
        ...(config.resolve?.alias ?? {}),
        react: pkg("react"),
        "react-dom": pkg("react-dom"),
        remotion: pkg("remotion"),
      },
    },
  }),
});
process.stdout.write(JSON.stringify({ bundle: out }) + "\n");
