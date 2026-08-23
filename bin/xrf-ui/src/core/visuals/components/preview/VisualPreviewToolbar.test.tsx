import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";

import { VisualPreviewToolbar } from "@/core/visuals/components/preview/VisualPreviewToolbar";
import {
  DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS,
  IVisualPreviewViewOptions,
} from "@/core/visuals/components/scene";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

function renderToolbar(
  detail: number,
  hasDetailLevels: boolean,
  onChangeDetail: (detail: number) => void = jest.fn()
): RenderResult {
  return renderWithProviders(
    <VisualPreviewToolbar
      options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
      isOpenEnabled
      detail={detail}
      hasDetailLevels={hasDetailLevels}
      hasSkeleton
      onChangeOptions={jest.fn()}
      onChangeDetail={onChangeDetail}
      onResetCamera={jest.fn()}
    />
  );
}

describe("VisualPreviewToolbar skeleton toggle", () => {
  it("offers nothing to draw on a model with no bind pose", () => {
    // Every model measured in gamedata carries one, so this state cannot be reached by opening a real file there -
    // which is exactly why it needs a test rather than a look.
    const render: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        isOpenEnabled
        detail={0}
        hasDetailLevels
        hasSkeleton={false}
        onChangeOptions={jest.fn()}
        onChangeDetail={jest.fn()}
        onResetCamera={jest.fn()}
      />
    );

    expect(render.getByRole("button", { name: "Bind pose skeleton" })).toBeDisabled();
  });

  it("asks for the overlay when a bind pose is there to draw", () => {
    const changes: Array<IVisualPreviewViewOptions> = [];
    const render: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        isOpenEnabled
        detail={0}
        hasDetailLevels
        hasSkeleton
        onChangeOptions={(options: IVisualPreviewViewOptions) => changes.push(options)}
        onChangeDetail={jest.fn()}
        onResetCamera={jest.fn()}
      />
    );

    fireEvent.click(render.getByRole("button", { name: "Bind pose skeleton" }));

    expect(changes).toHaveLength(1);
    expect(changes[0].isSkeletonVisible).toBe(true);
  });
});

describe("VisualPreviewToolbar detail control", () => {
  it("offers nothing to decimate on a model with one level", () => {
    const render: RenderResult = renderToolbar(0, false);

    expect(render.getByRole("button", { name: "Mesh detail" })).toBeDisabled();
  });

  it("reads as quality while reporting how far down the chain to go", async () => {
    // The slider is inverted against the stored value. Inverting only the label drew 25% quality as 25% decimation,
    // which looks like a working control until the triangle count is read.
    const changes: Array<number> = [];
    const render: RenderResult = renderToolbar(0, true, (detail: number) => changes.push(detail));

    fireEvent.click(render.getByRole("button", { name: "Mesh detail" }));

    const slider: Nullable<HTMLInputElement> = await waitFor(() =>
      document.querySelector<HTMLInputElement>('.MuiPopover-root input[type="range"]')
    );

    expect(slider).not.toBeNull();
    expect(slider!.value).toBe("100");

    fireEvent.change(slider!, { target: { value: "25" } });

    // A quarter of the quality is three quarters of the way down the chain.
    expect(changes).toEqual([0.75]);
  });

  it("shows the full mesh at the right of the slider", async () => {
    const render: RenderResult = renderToolbar(1, true);

    fireEvent.click(render.getByRole("button", { name: "Mesh detail" }));

    const slider: Nullable<HTMLInputElement> = await waitFor(() =>
      document.querySelector<HTMLInputElement>('.MuiPopover-root input[type="range"]')
    );

    // Coarsest stored detail is zero quality, so the handle sits at the left.
    expect(slider!.value).toBe("0");
  });
});
