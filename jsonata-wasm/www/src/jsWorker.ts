import jsonata, { JsonataError } from "jsonata";

self.onmessage = function (e: MessageEvent<string[]>) {
  try {
    // TODO: Think about whether JSON parsing of the input be included in the execution time
    const input = JSON.parse(e.data[1]);
    const start = performance.now();
    try {
      const j = jsonata(e.data[0]);
      const result = j.evaluate(input);
      const ms = Math.round((performance.now() - start) * 100) / 100;
      self.postMessage({ type: "success", ms, result: JSON.stringify(result) });
    } catch (e) {
      const err = e as JsonataError;
      self.postMessage({
        type: "failed",
        error: `${err.code} @ ${err.position}: ${err.message}`,
      });
    }
  } catch (e) {
    self.postMessage({
      type: "failed",
      error: `Failed to parse input: ${(e as Error).message}`,
    });
  }
};

export default self;
