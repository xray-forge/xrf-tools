import { describe, expect, it } from "@jest/globals";

import { groupDialogElements, IDialogElementGroup } from "@/applications/dialogs-editor/lib/dialog-elements";
import { DialogElementDescriptor } from "@/core/bindings/types/xrf-dialog";

function element(name: string, kind: DialogElementDescriptor["kind"], value: string = "x"): DialogElementDescriptor {
  return { kind, name, value };
}

function titlesOf(groups: Array<IDialogElementGroup>): Array<string> {
  return groups.map((group: IDialogElementGroup) => group.title);
}

describe("groupDialogElements", () => {
  it("groups nothing for a phrase carrying nothing", () => {
    expect(groupDialogElements([])).toEqual([]);
  });

  it("orders groups by the questions a reader asks, not by the file", () => {
    // An effect means nothing until you know what admits it, so gates come first however the file
    // happened to order them.
    const groups: Array<IDialogElementGroup> = groupDialogElements([
      element("init_func", "initFunc"),
      element("give_info", "giveInfo"),
      element("has_info", "hasInfo"),
    ]);

    expect(titlesOf(groups)).toEqual(["Conditions", "Effects", "Script"]);
  });

  it("returns only the groups with something in them", () => {
    // Four headings over one precondition would be three empty spaces and one fact.
    const groups: Array<IDialogElementGroup> = groupDialogElements([element("precondition", "precondition")]);

    expect(titlesOf(groups)).toEqual(["Conditions"]);
  });

  it("keeps a repeated element as repeated rows within its group", () => {
    // Two info gates are two conditions. One label with two values reads as one condition holding a
    // list, which is not what the engine does with them.
    const [conditions]: Array<IDialogElementGroup> = groupDialogElements([
      element("has_info", "hasInfo", "met"),
      element("has_info", "hasInfo", "armed"),
    ]);

    expect(conditions.elements.map((it: DialogElementDescriptor) => it.value)).toEqual(["met", "armed"]);
  });

  it("preserves document order inside a group", () => {
    const [conditions]: Array<IDialogElementGroup> = groupDialogElements([
      element("dont_has_info", "dontHasInfo", "first"),
      element("has_info", "hasInfo", "second"),
    ]);

    expect(conditions.elements.map((it: DialogElementDescriptor) => it.name)).toEqual(["dont_has_info", "has_info"]);
  });

  it("omits what the surrounding surface already shows", () => {
    // The header shows the line and the canvas draws the edges, so neither is a property here.
    expect(groupDialogElements([element("text", "text"), element("next", "next")])).toEqual([]);
  });

  it("does not report a schema element it has no group for as unrecognised", () => {
    // `is_final` is stated as a badge and `container` is structure the reader classified. Pooling them
    // into `Unrecognised` would accuse the schema of being off-schema.
    expect(groupDialogElements([element("is_final", "isFinal"), element("phrase_list", "container")])).toEqual([]);
  });

  it("keeps an off-schema element under its written name", () => {
    // One shipped project writes `go_back`, which the engine never defined.
    const [unrecognised]: Array<IDialogElementGroup> = groupDialogElements([element("go_back", "unknown", "1")]);

    expect(unrecognised.title).toBe("Unrecognised");
    expect(unrecognised.elements[0].name).toBe("go_back");
  });

  it("keeps a scripted line in the script group, not as content", () => {
    // The header has no line to show for a scripted phrase, so the reference is what identifies it.
    const [script]: Array<IDialogElementGroup> = groupDialogElements([
      element("script_text", "scriptText", "dialogs.trader_price"),
    ]);

    expect(script.title).toBe("Script");
    expect(script.elements[0].value).toBe("dialogs.trader_price");
  });
});
