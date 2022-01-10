import { defineConfig } from "vite";
import checker from "vite-plugin-checker";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  plugins: [checker({ typescript: true }), wasmPack("../../jsonata-wasm")],
});
