const esbuild = require("esbuild");

esbuild
  .build({
    entryPoints: ["src/tiptap-bridge.js"],
    bundle: true,
    format: "iife",
    outfile: "../assets/tiptap-bundle.js",
    minify: true,
    sourcemap: false,
  })
  .then(() => console.log("Built tiptap-bundle.js"))
  .catch(() => process.exit(1));
