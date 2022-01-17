import { Monaco } from "@monaco-editor/react";

function registerJsonata(monaco: Monaco) {
  monaco.languages.register({ id: "jsonata" });
  monaco.languages.setMonarchTokensProvider("jsonata", {
    keywords: ["true", "false", "null"],
    operators: [
      ".",
      "..",
      "<",
      ">",
      "<=",
      ">=",
      "=",
      "+",
      "-",
      "/",
      "%",
      "*",
      "**",
      "and",
      "or",
      "in",
      "~>",
      "@",
      "#",
      "&",
    ],
    tokenizer: {
      root: [
        [/\/\*.*\*\//, "comment"],
        [/'[^\\']'/, "string"],
        [/"[^\\"]"/, "string"],
        [/function/, "type.identifier"],
        [/[{}()\[\]]/, "@brackets"],
        [/\$[a-zA-Z0-9_]*/, "keyword"],
        [/\d*\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/\d+/, "number"],
        [
          /[a-zA-Z0-9_]+/,
          { cases: { "@operators": "operator", "@default": "identifier" } },
        ],
        [/;/, "delimiter"],
      ],
    },
  });

  const brackets = [
    { open: "(", close: ")" },
    { open: "[", close: "]" },
    { open: "{", close: "}" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
    { open: "`", close: "`" },
  ];
  monaco.languages.setLanguageConfiguration("jsonata", {
    brackets: [
      ["(", ")"],
      ["[", "]"],
      ["{", "}"],
    ],
    autoClosingPairs: brackets,
    surroundingPairs: brackets,
    indentationRules: {
      // ^(.*\*/)?\s*\}.*$
      decreaseIndentPattern: /^((?!.*?\/\*).*\*\/)?\s*[}\])].*$/,
      // ^.*\{[^}"']*$
      increaseIndentPattern:
        /^((?!\/\/).)*(\{[^}"'`]*|\([^)"'`]*|\[[^\]"'`]*)$/,
    },
  });
}

export default registerJsonata;
