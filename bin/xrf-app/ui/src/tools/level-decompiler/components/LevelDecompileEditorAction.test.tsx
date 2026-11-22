import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ELevelEditorField } from "@/core/level/lib/editor/level-editor-field";
import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelDecompileEditorAction } from "./LevelDecompileEditorAction";

const SCENE: string = "D:\\qa\\labx8\\rawdata\\levels\\labx8.level";

function remembered(field: ELevelEditorField): unknown {
  return getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_EDITOR, field));
}

describe("LevelDecompileEditorAction", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("opens the editor holding the scene, its own objects and the libraries it placed objects from", async () => {
    setLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_EDITOR, ELevelEditorField.GAME), '["C:\\\\cop"]');

    const { getByRole } = renderWithProviders(
      <LevelDecompileEditorAction scene={SCENE} objects={null} libraries={["C:\\SDK\\rawdata\\objects"]} />,
      { route: "/level-decompiler" }
    );

    await userEvent.click(getByRole("button", { name: "Open in editor" }));

    expect(remembered(ELevelEditorField.SCENE)).toBe(SCENE);
    expect(remembered(ELevelEditorField.OBJECTS)).toBe(
      JSON.stringify(["D:\\qa\\labx8\\rawdata\\objects", "C:\\SDK\\rawdata\\objects"])
    );
    expect(remembered(ELevelEditorField.GAME)).toBe('["C:\\\\cop"]');
  });

  // Roots remembered from another scene would look for this scene's objects elsewhere.
  it("empties the objects roots where the scene's objects lie beside it alone", async () => {
    setLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_EDITOR, ELevelEditorField.OBJECTS), '["D:\\\\other"]');

    const { getByRole } = renderWithProviders(
      <LevelDecompileEditorAction scene={SCENE} objects={null} libraries={[]} />,
      { route: "/level-decompiler" }
    );

    await userEvent.click(getByRole("button", { name: "Open in editor" }));

    expect(remembered(ELevelEditorField.OBJECTS)).toBeNull();
  });

  it("offers nothing before a scene was written", () => {
    const { getByRole } = renderWithProviders(
      <LevelDecompileEditorAction scene={null} objects={null} libraries={[]} />,
      {
        route: "/level-decompiler",
      }
    );

    expect(getByRole("button", { name: "Open in editor" })).toBeDisabled();
  });
});
