import { describe, expect, it } from "@jest/globals";

import { TranslationFile } from "@/core/ipc/types/xrf-translation";

import { TranslationDraft } from "./translation-draft";

const FILE: string = "st_test.json";

const COMMITTED: TranslationFile = {
  sources: {},
  entries: {
    scalar: { eng: "first\\nsecond" },
    lines: { eng: ["first", "second"] },
    absent: { eng: null },
  },
};

describe("TranslationDraft", () => {
  it("resolves committed values without treating them as edits", () => {
    const draft = TranslationDraft.empty();

    expect(draft.resolveValue(FILE, "eng", "scalar", "first\\nsecond")).toBe("first\\nsecond");
    expect(draft.resolveValue(FILE, "eng", "lines", ["first", "second"])).toBe("first\\nsecond");
    expect(draft.resolveValue(FILE, "eng", "empty", "")).toBe("");
    expect(draft.resolveValue(FILE, "eng", "absent", null)).toBeNull();
    expect(draft.hasEdit(FILE, "eng", "scalar")).toBe(false);
    expect(draft.dirtyFiles).toEqual([]);
    expect(draft.toFileEdits(FILE, COMMITTED)).toBeNull();
  });

  it.each([null, "", "changed"])("distinguishes a pending %s from an absent edit", (value) => {
    const draft = TranslationDraft.empty().withEdit(FILE, "eng", "scalar", value);

    expect(draft.resolveValue(FILE, "eng", "scalar", "committed")).toBe(value);
    expect(draft.hasEdit(FILE, "eng", "scalar")).toBe(true);
    expect(draft.dirtyFiles).toEqual([FILE]);
  });

  it("keeps previous drafts intact across edits and discard", () => {
    const empty = TranslationDraft.empty();
    const first = empty.withEdit(FILE, "eng", "scalar", "first edit");
    const second = first.withEdit(FILE, "eng", "scalar", "second edit");
    const discarded = second.withoutFile(FILE);

    expect(empty.resolveValue(FILE, "eng", "scalar", "committed")).toBe("committed");
    expect(first.resolveValue(FILE, "eng", "scalar", "committed")).toBe("first edit");
    expect(second.resolveValue(FILE, "eng", "scalar", "committed")).toBe("second edit");
    expect(discarded.resolveValue(FILE, "eng", "scalar", "committed")).toBe("committed");
    expect(discarded.dirtyFiles).toEqual([]);
  });

  it("isolates files, languages and ids while preserving dirty file order", () => {
    const draft = TranslationDraft.empty()
      .withEdit("second.json", "eng", "scalar", "other file")
      .withEdit(FILE, "eng", "scalar", "english")
      .withEdit(FILE, "rus", "scalar", "russian")
      .withEdit(FILE, "eng", "lines", "other id");
    const discarded = draft.withoutFile(FILE);

    expect(draft.dirtyFiles).toEqual(["second.json", FILE]);
    expect(draft.resolveValue(FILE, "eng", "scalar", null)).toBe("english");
    expect(draft.resolveValue(FILE, "rus", "scalar", null)).toBe("russian");
    expect(draft.resolveValue(FILE, "eng", "lines", null)).toBe("other id");
    expect(draft.hasEdit(FILE, "deu", "scalar")).toBe(false);
    expect(draft.hasEdit(FILE, "eng", "absent")).toBe(false);
    expect(discarded.dirtyFiles).toEqual(["second.json"]);
    expect(discarded.resolveValue("second.json", "eng", "scalar", null)).toBe("other file");
  });

  it("keeps an explicitly recorded committed value dirty", () => {
    const draft = TranslationDraft.empty().withEdit(FILE, "eng", "scalar", "first\\nsecond");

    expect(draft.resolveValue(FILE, "eng", "scalar", "first\\nsecond")).toBe("first\\nsecond");
    expect(draft.hasEdit(FILE, "eng", "scalar")).toBe(true);
    expect(draft.dirtyFiles).toEqual([FILE]);
    expect(draft.toFileEdits(FILE, COMMITTED)).toEqual({
      eng: [{ kind: "set", id: "scalar", value: "first\\nsecond" }],
    });
  });

  it("preserves variant shapes, literal separators, blank values and removals in save payloads", () => {
    const draft = TranslationDraft.empty()
      .withEdit(FILE, "eng", "scalar", "new\\ntext")
      .withEdit(FILE, "eng", "lines", "new\\ntext\\n")
      .withEdit(FILE, "eng", "absent", "")
      .withEdit(FILE, "eng", "removed", null)
      .withEdit(FILE, "rus", "lines", "new\\ntext")
      .withEdit(FILE, "rus", "actual-newline", "new\ntext");

    expect(draft.toFileEdits(FILE, COMMITTED)).toEqual({
      eng: [
        { kind: "set", id: "scalar", value: "new\\ntext" },
        { kind: "set", id: "lines", value: ["new", "text", ""] },
        { kind: "set", id: "absent", value: "" },
        { kind: "remove", id: "removed" },
      ],
      rus: [
        { kind: "set", id: "lines", value: "new\\ntext" },
        { kind: "set", id: "actual-newline", value: "new\ntext" },
      ],
    });
    expect(draft.dirtyFiles).toEqual([FILE]);
  });

  it("uses the committed shape supplied for each save without mutating an earlier payload", () => {
    const draft = TranslationDraft.empty().withEdit(FILE, "eng", "lines", "new\\ntext");
    const arrayPayload = draft.toFileEdits(FILE, COMMITTED);
    const scalarPayload = draft.toFileEdits(FILE, null);

    expect(arrayPayload).toEqual({ eng: [{ kind: "set", id: "lines", value: ["new", "text"] }] });
    expect(scalarPayload).toEqual({ eng: [{ kind: "set", id: "lines", value: "new\\ntext" }] });
    expect(COMMITTED.entries.lines.eng).toEqual(["first", "second"]);
  });
});
