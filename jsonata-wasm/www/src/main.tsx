import React, { useRef } from "react";
import ReactDOM from "react-dom";
import Editor, { Monaco } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { Layout, Model, TabNode } from "flexlayout-react";
import { atom, useAtom } from "jotai";
import { atomWithStorage, useAtomValue, useUpdateAtom } from "jotai/utils";
import jsonata, { JsonataError } from "jsonata";

import init, { evaluate } from "jsonata-wasm";
import "flexlayout-react/style/dark.css";
import "@picocss/pico/css/pico.css";
import "./style.css";
import demo from "./demo.json";
import jsonataMode from "./jsonataMonaco";

const defaultExpr = "$sum(Account.Order.Product.(Price * Quantity))";

await init();

const layoutModelAtom = atom<Model>(
  Model.fromJson({
    global: {
      tabEnableClose: false,
      tabEnableDrag: false,
      tabSetEnableDeleteWhenEmpty: false,
    },
    borders: [],
    layout: {
      type: "row",
      weight: 100,
      children: [
        {
          type: "row",
          weight: 60,
          children: [
            {
              type: "tabset",
              id: "expressions",
              children: [
                {
                  type: "tab",
                  name: "Untitled",
                  enableClose: true,
                  component: "expression",
                },
              ],
            },
            {
              type: "tabset",
              children: [
                {
                  type: "tab",
                  name: "Input",
                  component: "input",
                },
              ],
            },
          ],
        },
        {
          type: "row",
          weight: 40,
          children: [
            {
              type: "tabset",
              children: [
                {
                  type: "tab",
                  name: "jsonata-rust",
                  component: "outputRust",
                },
              ],
            },
            {
              type: "tabset",
              children: [
                {
                  type: "tab",
                  name: "jsonata",
                  component: "outputJs",
                },
              ],
            },
          ],
        },
      ],
    },
  })
);

const inputAtom = atom(JSON.stringify(demo, null, 2));

const outputRustAtom = atom(
  "Run an expression with Ctrl/Cmd+Enter to see output..."
);

const outputJsAtom = atom(
  "Run an expression with Ctrl/Cmd+Enter to see output..."
);

const Container: React.FC = (props) => {
  return <div className="main-container">{props.children}</div>;
};

const Header: React.FC<{ newExpression: () => void }> = (props) => {
  return (
    <div className="header">
      <a href="#" onClick={props.newExpression}>
        New Expression
      </a>
    </div>
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
        setRustOutput(evaluate(expr, inputRef.current));
      } catch (e) {
        setRustOutput("Failed: " + e);
      }

      //
      // Js result
      //
      try {
        const input = JSON.parse(inputRef.current);
        try {
          const j = jsonata(expr);
          const result = j.evaluate(JSON.parse(inputRef.current));
          setJsOutput(JSON.stringify(result));
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
  const [layoutModel, setLayoutModel] = useAtom(layoutModelAtom);
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
      <Header newExpression={newExpression} />
      <div className="layout">
        <Layout ref={layoutRef} model={layoutModel} factory={layoutFactory} />
      </div>
    </Container>
  );
}

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById("root")
);
