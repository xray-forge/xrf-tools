import { describe, expect, it } from "@jest/globals";

import {
  EXTERN_MANIFEST_FORMATS,
  withExternManifestExtension,
} from "@/applications/exports-explorer/lib/extern-manifest";

describe("withExternManifestExtension", () => {
  it("keeps a destination naming a format the backend reads", () => {
    for (const format of EXTERN_MANIFEST_FORMATS) {
      expect(withExternManifestExtension(`C:\\out\\extern.${format}`)).toBe(`C:\\out\\extern.${format}`);
    }

    // Offered by nothing and typed by hand, but the backend renders HTML for it.
    expect(withExternManifestExtension("C:\\out\\extern.htm")).toBe("C:\\out\\extern.htm");
  });

  it("reads the format case-insensitively, as a save dialog returns it", () => {
    expect(withExternManifestExtension("C:\\out\\EXTERN.XML")).toBe("C:\\out\\EXTERN.XML");
  });

  it("names the default format when the destination names none", () => {
    expect(withExternManifestExtension("C:\\out\\extern")).toBe("C:\\out\\extern.json");
    expect(withExternManifestExtension("C:\\out\\extern.txt")).toBe("C:\\out\\extern.txt.json");
  });

  it("reads the last segment, so a dotted directory decides nothing", () => {
    expect(withExternManifestExtension("C:\\out.xml\\extern")).toBe("C:\\out.xml\\extern.json");
  });
});
