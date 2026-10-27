import { describe, expect, it } from "@jest/globals";

import { toChoiceListFormRowSections } from "#/form/ChoiceListFormRow/ChoiceListFormRow.utils";

describe("toChoiceListFormRowSections", () => {
  it("keeps options without a section as one bare run", () => {
    expect(
      toChoiceListFormRowSections([
        { label: "a", value: "a" },
        { label: "b", value: "b" },
      ])
    ).toEqual([
      {
        label: undefined,
        options: [
          { label: "a", value: "a" },
          { label: "b", value: "b" },
        ],
      },
    ]);
  });

  // The list keeps the order offered: a section listed apart from its run is headed again rather than regrouped.
  it("starts another run where a section recurs apart from its first", () => {
    const sections = toChoiceListFormRowSections([
      { label: "a", value: "a", section: "One" },
      { label: "b", value: "b", section: "Two" },
      { label: "c", value: "c", section: "One" },
    ]);

    expect(sections.map((section) => [section.label, section.options.map((option) => option.value)])).toEqual([
      ["One", ["a"]],
      ["Two", ["b"]],
      ["One", ["c"]],
    ]);
  });
});
