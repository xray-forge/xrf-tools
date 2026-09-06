import { describe, expect, it, jest } from "@jest/globals";

import {
  ETextureOpenMode,
  getTextureOpenMode,
  ITextureOpenModeDescriptor,
  ITextureOpenSession,
  TEXTURE_OPEN_MODES,
} from "@/applications/textures-explorer/lib/texture-open-mode";

const PATH: string = "C:\\gamedata";

function mockSession(): ITextureOpenSession & {
  calls: Array<string>;
} {
  const calls: Array<string> = [];

  return {
    calls,
    catalogService: {
      close: jest.fn(() => void calls.push("close")),
      openLooseDirectory: jest.fn(() => void calls.push("openLooseDirectory")),
      openRoot: jest.fn(() => void calls.push("openRoot")),
    },
    selectionService: { openFile: jest.fn(() => void calls.push("openFile")) },
  } as unknown as ITextureOpenSession & { calls: Array<string> };
}

describe("TEXTURE_OPEN_MODES", () => {
  it("offers every way in exactly once, in toggle order", () => {
    expect(TEXTURE_OPEN_MODES.map((it: ITextureOpenModeDescriptor) => it.id)).toEqual([
      ETextureOpenMode.FOLDER,
      ETextureOpenMode.LOOSE_FOLDER,
      ETextureOpenMode.TEXTURE,
    ]);
  });

  it("says what each mode reads and what its button does", () => {
    // The facts the form used to spell in eight places. A mode missing one of them renders a blank label or a row
    // with no description, which is the drift the table exists to prevent.
    for (const mode of TEXTURE_OPEN_MODES) {
      expect(mode.label).toBeTruthy();
      expect(mode.description).toBeTruthy();
      expect(mode.submitLabel).toBeTruthy();
      expect(mode.field.label).toBeTruthy();
      expect(mode.field.description).toBeTruthy();
    }
  });

  it("browses a game tree by its roots", async () => {
    const session = mockSession();

    await getTextureOpenMode(ETextureOpenMode.FOLDER).open(PATH, session);

    expect(session.calls).toEqual(["openRoot"]);
  });

  it("browses a loose folder by its path", async () => {
    const session = mockSession();

    await getTextureOpenMode(ETextureOpenMode.LOOSE_FOLDER).open(PATH, session);

    expect(session.calls).toEqual(["openLooseDirectory"]);
  });

  it("closes the browsed session before opening one texture on its own", async () => {
    // Order is the point: a texture from a previous root has nothing to do with the file being opened, and a tree
    // left on screen beside it would be the disagreement the explorer exists to prevent.
    const session = mockSession();

    await getTextureOpenMode(ETextureOpenMode.TEXTURE).open(PATH, session);

    expect(session.calls).toEqual(["close", "openFile"]);
  });
});
