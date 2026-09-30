import { describe, expect, it } from "@jest/globals";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";

import { toFormatFilters } from "./file-filters";

describe("toFormatFilters", () => {
  it("offers one entry per format, in order, each named from its format", () => {
    expect(toFormatFilters([EXrayExtension.JSON, EXrayExtension.XML], (format) => `${format} manifest`)).toEqual([
      { extensions: ["json"], name: "json manifest" },
      { extensions: ["xml"], name: "xml manifest" },
    ]);
  });
});
