import { describe, expect, it } from "@jest/globals";

import { toVocabularyOptions } from "./texture-vocabulary";

const FORMATS = [
  { label: "tfDXT1", value: 0 },
  { label: "tfDXT5", value: 4 },
];

describe("toVocabularyOptions", () => {
  it("should offer only the named values when the stored one is among them", () => {
    expect(toVocabularyOptions(FORMATS, 4)).toEqual(FORMATS);
  });

  it("should keep a value the sdk never named visible rather than dropping to the first entry", () => {
    // A newer converter or a file edited by hand can hold one, and moving it silently is how an editor corrupts
    // somebody else's descriptor. The label names the number so a person can see what they are keeping.
    expect(toVocabularyOptions(FORMATS, 7)).toEqual([...FORMATS, { label: "Unknown (7)", value: 7 }]);
  });
});
