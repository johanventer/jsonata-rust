import init, { evaluate } from "jsonata-wasm";

await init();

self.onmessage = function (e: MessageEvent<string[]>) {
  try {
    const start = performance.now();
    const result = evaluate(e.data[0], e.data[1]);
    const ms = Math.round((performance.now() - start) * 100) / 100;
    self.postMessage({ type: "success", ms, result });
  } catch (e) {
    self.postMessage({ type: "failed", error: e as string });
  }
};

export default self;
