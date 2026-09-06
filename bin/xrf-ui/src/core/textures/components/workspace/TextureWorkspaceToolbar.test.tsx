import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { TextureWorkspaceToolbar } from "@/core/textures/components/workspace/TextureWorkspaceToolbar";
import {
  DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  ETexturePreviewMode,
  ITexturePreviewOptions,
} from "@/core/textures/lib/texture-preview";
import { renderWithProviders } from "@/fixtures/utils/render";

const SURFACE: ITexturePreviewOptions = { ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, mode: ETexturePreviewMode.SURFACE };

function renderToolbar(
  options: ITexturePreviewOptions = DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  hasBump: boolean = true,
  onChangeOptions: (options: ITexturePreviewOptions) => void = jest.fn()
): RenderResult {
  return renderWithProviders(
    <TextureWorkspaceToolbar
      location={null}
      options={options}
      hasBump={hasBump}
      onChangeOptions={onChangeOptions}
      onResetCamera={jest.fn()}
      onBack={jest.fn()}
    />
  );
}

describe("TextureWorkspaceToolbar", () => {
  it("offers nothing about a body while the flat picture is on screen", () => {
    const { getByLabelText } = renderToolbar();

    expect((getByLabelText("Body") as HTMLButtonElement).disabled).toBe(true);
    expect((getByLabelText("Light") as HTMLButtonElement).disabled).toBe(true);
    expect((getByLabelText("Bump") as HTMLButtonElement).disabled).toBe(true);
    expect((getByLabelText("Reset camera") as HTMLButtonElement).disabled).toBe(true);
  });

  it("switches between the picture and the lit body from one control", () => {
    const onChangeOptions = jest.fn();
    const { getByLabelText } = renderToolbar(DEFAULT_TEXTURE_PREVIEW_OPTIONS, true, onChangeOptions);

    fireEvent.click(getByLabelText("Lit surface"));

    expect(onChangeOptions).toHaveBeenCalledWith({ ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, mode: "surface" });
  });

  it("opens the body controls once there is a body to set up", () => {
    const onChangeOptions = jest.fn();
    const { getByLabelText, getByText } = renderToolbar(SURFACE, true, onChangeOptions);

    fireEvent.click(getByLabelText("Body"));
    fireEvent.click(getByText("Sphere"));

    expect(onChangeOptions).toHaveBeenCalledWith({ ...SURFACE, shape: "sphere" });

    fireEvent.click(getByLabelText("Tile 4 by 4"));

    expect(onChangeOptions).toHaveBeenCalledWith({ ...SURFACE, tiling: 4 });
  });

  it("refuses to shade with a pair the texture does not declare", () => {
    const { getByLabelText } = renderToolbar(SURFACE, false);

    expect((getByLabelText("Bump") as HTMLButtonElement).disabled).toBe(true);
  });

  it("refuses to shade with a pair when there is no light to shade by", () => {
    // Under a flat ambient the decoded normal changes no pixel, so offering the switch would say something untrue.
    const { getByLabelText } = renderToolbar({ ...SURFACE, isLit: false });

    expect((getByLabelText("Bump") as HTMLButtonElement).disabled).toBe(true);
    expect((getByLabelText("Light") as HTMLButtonElement).disabled).toBe(false);
  });
});
