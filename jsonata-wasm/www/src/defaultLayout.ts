import { IJsonModel } from "flexlayout-react";

export default {
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
} as IJsonModel;
