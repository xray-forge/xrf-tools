import { describe, expect, it } from "@jest/globals";

import { buildSearchIndex, ISearchIndexEntry, rankedSearch } from "@/core/search/lib/ranked-search";

interface IFile {
  name: string;
}

function indexOf(names: Array<string>): Array<ISearchIndexEntry<IFile>> {
  return buildSearchIndex(
    names.map((name: string) => ({ name })),
    (file: IFile) => file.name
  );
}

function getNamesOf(names: Array<string>, query: string, limit: number = 50): Array<string> {
  return rankedSearch(indexOf(names), query, limit).results.map((result) => result.name);
}

describe("rankedSearch", () => {
  it("puts the closest match first rather than the alphabetically earliest", () => {
    // Ordering is what makes capping safe: alphabetically, `aaa_dialogs_old` wins and the file the
    // user actually named never appears once the limit bites.
    const ranked: Array<string> = getNamesOf(
      ["configs\\aaa_dialogs_old.xml", "dialogs.xml", "configs\\dialogs_zaton.xml", "st_dialogs_jupiter.json"],
      "dialogs"
    );

    expect(ranked[0]).toBe("dialogs.xml");
    expect(ranked.indexOf("configs\\dialogs_zaton.xml")).toBeLessThan(ranked.indexOf("st_dialogs_jupiter.json"));
  });

  it("prefers a file name match over one buried in a directory", () => {
    const ranked: Array<string> = getNamesOf(["dialogs\\other.xml", "configs\\dialogs.xml"], "dialogs");

    expect(ranked[0]).toBe("configs\\dialogs.xml");
  });

  it("prefers an exact file name even when its directory matches first", () => {
    const ranked: Array<string> = getNamesOf(["configs/dialogs_extra.xml", "dialogs/dialogs.xml"], "dialogs");

    expect(ranked).toEqual(["dialogs/dialogs.xml", "configs/dialogs_extra.xml"]);
  });

  it("prefers a segment match even when an earlier occurrence is inside a word", () => {
    const ranked: Array<string> = getNamesOf(["aaa_mydialogs.xml", "zzz_mydialogs.dialogs.xml"], "dialogs");

    expect(ranked).toEqual(["zzz_mydialogs.dialogs.xml", "aaa_mydialogs.xml"]);
  });

  it("reports every match while returning only the limit", () => {
    const names: Array<string> = Array.from({ length: 500 }, (_, index) => `configs\\file_${index}_dialogs.xml`);

    const outcome = rankedSearch(indexOf(names), "dialogs", 200);

    expect(outcome.results).toHaveLength(200);
    // The count has to be truthful, or "showing 200 of N" lies about what was left out.
    expect(outcome.total).toBe(500);
  });

  it("matches case-insensitively without lowercasing per query", () => {
    expect(getNamesOf(["Configs\\DIALOGS.xml"], "dialogs")).toHaveLength(1);
    expect(getNamesOf(["configs\\dialogs.xml"], "DIALOGS")).toHaveLength(1);
  });

  it("returns nothing for an empty or whitespace query", () => {
    expect(rankedSearch(indexOf(["a.xml"]), "", 10).results).toHaveLength(0);
    expect(rankedSearch(indexOf(["a.xml"]), "   ", 10).total).toBe(0);
  });

  it("sorts a documentation-only match below every name match", () => {
    // Exports search their descriptions too. Ranking those on the same footing as a name would let a
    // word buried in a parameter doc outrank the declaration the user actually typed.
    const index = buildSearchIndex<IFile>(
      [{ name: "unrelated.helper" }, { name: "xr_effects.play_sound" }],
      (file: IFile) => file.name,
      (file: IFile) => (file.name === "unrelated.helper" ? "plays a sound for the actor" : "")
    );

    const outcome = rankedSearch(index, "sound", 10);

    expect(outcome.total).toBe(2);
    expect(outcome.results.map((result) => result.name)).toEqual(["xr_effects.play_sound", "unrelated.helper"]);
  });

  it("returns the original item so selection retains its identity", () => {
    const file: IFile = { name: "configs\\dialogs.xml" };
    const index = buildSearchIndex([file], (item: IFile) => item.name);
    const outcome = rankedSearch(index, "dialogs", 10);

    expect(outcome.results[0]).toBe(file);
  });
});
