import React, { useRef } from "react";
import ReactDOM from "react-dom";
import Editor, { Monaco } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { Layout, Model, TabNode } from "flexlayout-react";
import { atom, useAtom } from "jotai";
import { atomWithStorage, useAtomValue, useUpdateAtom } from "jotai/utils";
import jsonata, { JsonataError } from "jsonata";

import init, { evaluate } from "jsonata-wasm";
import demo from "./demo.json";
import defaultLayout from "./defaultLayout";
import jsonataMode from "./jsonataMonaco";

import "flexlayout-react/style/dark.css";
import "./style.css";

const defaultExpr = "$sum(Account.Order.Product.(Price * Quantity))";

const inputAtom = atomWithStorage("input", JSON.stringify(demo, null, 2));
const outputRustAtom = atom(
  "Run an expression with Ctrl/Cmd+Enter to see output..."
);
const outputJsAtom = atom(
  "Run an expression with Ctrl/Cmd+Enter to see output..."
);

const Container: React.FC = (props) => {
  return <div className="main-container">{props.children}</div>;
};

const Toolbar: React.FC<{ newExpression: () => void }> = (props) => {
  const setInput = useUpdateAtom(inputAtom);

  return (
    <ul className="toolbar">
      <li>
        <a href="#" onClick={props.newExpression}>
          New Expression
        </a>
      </li>
      <li>
        <a href="#" onClick={() => setInput(JSON.stringify(demo, null, 2))}>
          Default Input
        </a>
      </li>
    </ul>
  );
};

const ExpressionEditor: React.FC<{
  handleRun: (expr: string | undefined) => void;
}> = (props) => {
  const input = useAtomValue(inputAtom);
  const inputRef = useRef(input);
  const setRustOutput = useUpdateAtom(outputRustAtom);
  const setJsOutput = useUpdateAtom(outputJsAtom);

  // Need to capture the latest input in a ref due to the addCommand closure
  inputRef.current = input;

  function handleBeforeMount(monaco: Monaco) {
    jsonataMode(monaco);
  }

  function handleMount(
    editor: monaco.editor.IStandaloneCodeEditor,
    monaco: Monaco
  ) {
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
      setRustOutput("Evaluating...");
      setJsOutput("Evaluating...");

      const expr = editor.getValue();

      //
      // Rust result
      //
      try {
        const start = performance.now();
        const result = evaluate(expr, inputRef.current);
        const ms = Math.round((performance.now() - start) * 100) / 100;
        setRustOutput(`Execution time: ${ms}ms\n\nResult:\n\n${result}`);
      } catch (e) {
        setRustOutput("Failed: " + e);
      }

      //
      // Js result
      //
      try {
        const start = performance.now();
        // NOTE: Include the JSON parsing of the input as the Rust side has to do that as well.
        const input = JSON.parse(inputRef.current);
        try {
          const j = jsonata(expr);
          const result = j.evaluate(input);
          const ms = Math.round((performance.now() - start) * 100) / 100;
          setJsOutput(
            `Execution time: ${ms}ms\n\nResult:\n\n${JSON.stringify(result)}`
          );
        } catch (e) {
          const err = e as JsonataError;
          setJsOutput(`${err.code} @ ${err.position}: ${err.message}`);
        }
      } catch (e) {
        setJsOutput("Failed to parse input: " + (e as Error).message);
      }
    });
  }

  return (
    <div className="editor-wrapper">
      <Editor
        theme="vs-dark"
        language="jsonata"
        value={defaultExpr}
        options={{
          automaticLayout: true,
          contextmenu: false,
          minimap: { enabled: false },
        }}
        beforeMount={handleBeforeMount}
        onMount={handleMount}
      />
    </div>
  );
};

const InputEditor: React.FC = (props) => {
  const [input, setInput] = useAtom(inputAtom);
  return (
    <div className="editor-wrapper">
      <Editor
        theme="vs-dark"
        defaultLanguage="json"
        options={{
          automaticLayout: true,
          contextmenu: false,
          minimap: { enabled: false },
        }}
        value={input}
        onChange={(v) => (v !== undefined ? setInput(v) : setInput("{}"))}
      />
    </div>
  );
};

const OutputRust: React.FC = (props) => {
  const outputRust = useAtomValue(outputRustAtom);

  return (
    <div className="editor-wrapper">
      <div className="output">{outputRust}</div>
    </div>
  );
};

const OutputJs: React.FC = (props) => {
  const outputJs = useAtomValue(outputJsAtom);

  return (
    <div className="editor-wrapper">
      <div className="output">{outputJs}</div>
    </div>
  );
};

function App() {
  const layoutRef = useRef<Layout | null>(null);

  function handleRun(expr: string | undefined) {}

  function layoutFactory(node: TabNode) {
    switch (node.getComponent()) {
      case "expression":
        return <ExpressionEditor handleRun={handleRun} />;
      case "input":
        return <InputEditor />;
      case "outputRust":
        return <OutputRust />;
      case "outputJs":
        return <OutputJs />;
      default:
        return <div>Unknown</div>;
    }
  }

  function newExpression() {
    layoutRef.current?.addTabToTabSet("expressions", {
      type: "tab",
      name: "Untitled",
      enableClose: true,
      component: "expression",
    });
  }

  return (
    <Container>
      <Toolbar newExpression={newExpression} />
      <div className="layout">
        <Layout
          ref={layoutRef}
          model={Model.fromJson(defaultLayout)}
          factory={layoutFactory}
        />
      </div>
    </Container>
  );
}

await init();

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById("root")
);
