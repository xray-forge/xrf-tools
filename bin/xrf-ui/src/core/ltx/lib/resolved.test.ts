import { describe, expect, it, jest } from "@jest/globals";

import {
  LtxResolvedField,
  LtxResolvedFieldOrigin,
  LtxResolvedIndexEntry,
  LtxResolvedSection,
} from "@/core/bindings/types/xrf-ltx-inspect";
import { describeResolvedFieldOrigin, IResolvedLayout, toResolvedLayout } from "@/core/ltx/lib/resolved";
import { ESyntaxToken, ISyntaxSpan } from "@/core/syntax/lib";
import { ICodeLineSource } from "@/core/ui/code/code-line";

const ENTRY: string = "configs\\system.ltx";

/** An index entry carrying only what a case is about. */
function entryOf(name: string, fieldCount: number, parents: Array<string> = []): LtxResolvedIndexEntry {
  return { fieldCount, name, origin: "configs\\weapons.ltx", parents };
}

/** A field written in the section that holds it, which is the case most of a config is. */
function fieldOf(key: string, value: string, origin?: LtxResolvedFieldOrigin): LtxResolvedField {
  return { key, origin: origin ?? { file: "configs\\weapons.ltx", kind: "declared" }, value };
}

/** A fetched body for one section. */
function bodyOf(name: string, fields: Array<LtxResolvedField>, parents: Array<string> = []): LtxResolvedSection {
  return { entry: ENTRY, fields, name, origin: "configs\\weapons.ltx", parents };
}

/** A layout over the given sections, read through a source holding whatever bodies a case supplies. */
function readingOf(
  sections: Array<LtxResolvedIndexEntry>,
  bodies?: ReadonlyMap<string, LtxResolvedSection>
): { layout: IResolvedLayout; source: ICodeLineSource } {
  const layout: IResolvedLayout = toResolvedLayout(sections);

  return { layout, source: layout.toSource(bodies) };
}

/** Every span of one line, in order, as `token:text` pairs a failure can be read from. */
function spansOf(source: ICodeLineSource, line: number): Array<string> {
  return source.getLine(line - 1).spans.map((span: ISyntaxSpan) => `${span.token}:${span.text}`);
}

/** The text of one line, which is what a reader sees whatever it is coloured. */
function textOf(source: ICodeLineSource, line: number): string {
  return source
    .getLine(line - 1)
    .spans.map((span: ISyntaxSpan) => span.text)
    .join("");
}

describe("toResolvedLayout", () => {
  it("should take its height from the index alone, before a single body has arrived", () => {
    // The whole reason the index carries a field count: the document is as tall as it will ever be on the first
    // answer, so scrolling it does not move under the person scrolling.
    const { layout, source } = readingOf([entryOf("wpn_ak74", 3), entryOf("wpn_lr300", 1)]);

    expect(layout.lineCount).toBe(3 + 2 + 1 + 2);
    expect(Array.from({ length: layout.lineCount }, (_, at: number) => source.getLine(at).number)).toEqual([
      1, 2, 3, 4, 5, 6, 7, 8,
    ]);
  });

  it("should build a line only when one is asked for", () => {
    // What the whole shape is for: Anomaly's `system.ltx` resolves to 551,000 lines and a page of bodies lands on
    // every scroll, so a layout that read them - or built them - would cost the document per page.
    const bodies: Map<string, LtxResolvedSection> = new Map([["s500", bodyOf("s500", [fieldOf("cost", "1")])]]);
    const reads = jest.spyOn(bodies, "get");
    const { layout, source } = readingOf(
      Array.from({ length: 1000 }, (_, at: number) => entryOf(`s${at}`, 10)),
      bodies
    );

    expect(layout.lineCount).toBe(1000 * 12);
    expect(reads).not.toHaveBeenCalled();

    source.getLine(0);
    source.getLine(1);

    expect(reads).toHaveBeenCalledTimes(1);
  });

  it("should keep every line where it was once bodies arrive", () => {
    const sections: Array<LtxResolvedIndexEntry> = [entryOf("wpn_ak74", 2), entryOf("wpn_lr300", 1)];
    const empty = readingOf(sections);
    const filled = readingOf(sections, new Map([["wpn_lr300", bodyOf("wpn_lr300", [fieldOf("cost", "1000")])]]));

    expect(filled.layout.lineCount).toBe(empty.layout.lineCount);
    expect(filled.layout.getSectionLine("wpn_lr300")).toBe(empty.layout.getSectionLine("wpn_lr300"));
    expect(textOf(filled.source, 5)).toBe("[wpn_lr300]");
  });

  it("should put a fetched section's fields on the lines the index predicted", () => {
    const { source } = readingOf(
      [entryOf("wpn_ak74", 2, ["wpn_base"])],
      new Map([["wpn_ak74", bodyOf("wpn_ak74", [fieldOf("cost", "1000"), fieldOf("ammo_mag_size", "30")])]])
    );

    expect(textOf(source, 1)).toBe("[wpn_ak74]:wpn_base");
    expect(textOf(source, 2)).toBe("cost = 1000  ; written here, in configs\\weapons.ltx");
    expect(textOf(source, 3)).toBe("ammo_mag_size = 30  ; written here, in configs\\weapons.ltx");
    expect(textOf(source, 4)).toBe("");
  });

  it("should render a section with no body yet as blank lines that later fill in", () => {
    const { source } = readingOf([entryOf("wpn_ak74", 2)]);

    expect(spansOf(source, 1)).toEqual([`${ESyntaxToken.SECTION}:[wpn_ak74]`]);
    expect(source.getLine(1).spans).toEqual([]);
    expect(source.getLine(2).spans).toEqual([]);
  });

  it("should colour a field with the shared token vocabulary and dim the note it adds", () => {
    const { source } = readingOf(
      [entryOf("wpn_ak74", 1)],
      new Map([["wpn_ak74", bodyOf("wpn_ak74", [fieldOf("cost", "1000", { kind: "unrecorded" })])]])
    );

    expect(spansOf(source, 2)).toEqual([
      `${ESyntaxToken.KEY}:cost`,
      `${ESyntaxToken.PLAIN}: `,
      `${ESyntaxToken.OPERATOR}:=`,
      `${ESyntaxToken.PLAIN}: 1000`,
      `${ESyntaxToken.COMMENT}:  ; origin not recorded`,
    ]);
  });

  it("should separate the parents of a header with the operators the file spells them with", () => {
    const { source } = readingOf([entryOf("wpn_ak74", 0, ["wpn_base", "wpn_ammo"])]);

    expect(spansOf(source, 1)).toEqual([
      `${ESyntaxToken.SECTION}:[wpn_ak74]`,
      `${ESyntaxToken.OPERATOR}::`,
      `${ESyntaxToken.TYPE}:wpn_base`,
      `${ESyntaxToken.OPERATOR}:,`,
      `${ESyntaxToken.TYPE}:wpn_ammo`,
    ]);
  });

  it("should answer the sections a window of lines covers", () => {
    // Two fields each, so the sections start on lines 1, 5 and 9.
    const { layout } = readingOf([entryOf("a", 2), entryOf("b", 2), entryOf("c", 2)]);

    expect(layout.getSectionsInRange(1, 3)).toEqual(["a"]);
    expect(layout.getSectionsInRange(3, 6)).toEqual(["a", "b"]);
    expect(layout.getSectionsInRange(1, 12)).toEqual(["a", "b", "c"]);
    // A window resting on the gap after `b`, which is a line `b` owns.
    expect(layout.getSectionsInRange(8, 8)).toEqual(["b"]);
    expect(layout.getSectionsInRange(13, 20)).toEqual([]);
    expect(layout.getSectionsInRange(6, 2)).toEqual([]);
  });

  it("should find the line a named section starts on, and refuse to invent one", () => {
    const { layout } = readingOf([entryOf("a", 3), entryOf("b", 1)]);

    expect(layout.getSectionLine("a")).toBe(1);
    expect(layout.getSectionLine("b")).toBe(6);
    expect(layout.getSectionLine("missing")).toBeNull();
  });

  it("should keep the order the dialect answered in, whatever that order is", () => {
    // Authored order under standard LTX, name order under DLTX. Both are the engine's own output.
    const { layout, source } = readingOf([entryOf("zzz", 0), entryOf("aaa", 0)]);

    expect([textOf(source, 1), textOf(source, 3)]).toEqual(["[zzz]", "[aaa]"]);
    expect(layout.getSectionsInRange(1, 4)).toEqual(["zzz", "aaa"]);
  });

  it("should name the root section for what it is rather than calling it a section", () => {
    // The unnamed section holds what was written before the first header, and `[]` is a header nobody wrote.
    const { source } = readingOf([entryOf("", 1)], new Map([["", bodyOf("", [fieldOf("mp_maps", "mp_pool")])]]));

    expect(spansOf(source, 1)).toEqual([`${ESyntaxToken.COMMENT}:; fields written before any section header`]);
    // Its per-field file is the first config merged rather than the one the field is written in, so it is not claimed.
    expect(textOf(source, 2)).toBe("mp_maps = mp_pool  ; written here");
  });

  it("should ignore a body for a section the index does not list", () => {
    const { layout, source } = readingOf([entryOf("a", 1)], new Map([["b", bodyOf("b", [fieldOf("cost", "1")])]]));

    expect(layout.lineCount).toBe(3);
    expect(source.getLine(1).spans).toEqual([]);
  });

  it("should stop at the field count the index gave when a body disagrees with it", () => {
    // The two come from one resolution and cannot really disagree; if they ever did, the layout is what must not move.
    const { layout, source } = readingOf(
      [entryOf("a", 1)],
      new Map([["a", bodyOf("a", [fieldOf("cost", "1"), fieldOf("weight", "2")])]])
    );

    expect(layout.lineCount).toBe(3);
    expect(textOf(source, 2)).toBe("cost = 1  ; written here, in configs\\weapons.ltx");
  });

  it("should answer nothing for a line the document does not hold", () => {
    // A listing asks for the window it last rendered, which can outlive the document that was that tall.
    const { source } = readingOf([entryOf("a", 1)]);

    expect(source.getLine(99).spans).toEqual([]);
    expect(source.indexOfLine(99)).toBe(-1);
    expect(source.indexOfLine(2)).toBe(1);
  });
});

describe("describeResolvedFieldOrigin", () => {
  it("should say a field written here, with the config it is written in when one was recorded", () => {
    expect(describeResolvedFieldOrigin({ file: "configs\\w_ak74.ltx", kind: "declared" })).toBe(
      "written here, in configs\\w_ak74.ltx"
    );
    expect(describeResolvedFieldOrigin({ file: null, kind: "declared" })).toBe("written here");
  });

  it("should name the section an inherited field is written in, which is not the parent in the header", () => {
    expect(describeResolvedFieldOrigin({ file: "configs\\w_base.ltx", kind: "inherited", section: "wpn_base" })).toBe(
      "inherited from [wpn_base] in configs\\w_base.ltx"
    );
    expect(describeResolvedFieldOrigin({ file: null, kind: "inherited", section: "wpn_base" })).toBe(
      "inherited from [wpn_base]"
    );
  });

  it("should name the operation and depth a patched field won on", () => {
    expect(
      describeResolvedFieldOrigin({ depth: 2, file: "configs\\mod_system_zzz.ltx", kind: "loaded", operation: "!" })
    ).toBe("set by configs\\mod_system_zzz.ltx ('!', depth 2)");
    expect(describeResolvedFieldOrigin({ depth: 0, file: "configs\\system.ltx", kind: "loaded", operation: "" })).toBe(
      "set by configs\\system.ltx (depth 0)"
    );
  });

  it("should say an unrecorded origin rather than guessing one", () => {
    // A resolution produced without asking for provenance. Inventing "written here" for it is the one wrong answer.
    expect(describeResolvedFieldOrigin({ kind: "unrecorded" })).toBe("origin not recorded");
  });

  it("should not claim a config for a root field, whose recorded file is the first one merged", () => {
    expect(describeResolvedFieldOrigin({ file: "configs\\system.ltx", kind: "declared" }, true)).toBe("written here");
  });
});
