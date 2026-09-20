import { describe, expect, it, jest } from "@jest/globals";
import { popoverClasses } from "@mui/material";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";

import { VisualPreviewToolbar } from "@/core/visuals/components/preview/VisualPreviewToolbar";
import {
  DEFAULT_VISUAL_LIGHTING,
  DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS,
  IVisualPreviewViewOptions,
} from "@/core/visuals/lib/scene";
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
      lighting={DEFAULT_VISUAL_LIGHTING}
      detail={detail}
      hasDetailLevels={hasDetailLevels}
      hasSkeleton={true}
      hasBump={true}
      hasAlpha={true}
      onChangeOptions={jest.fn()}
      onChangeLighting={jest.fn()}
      onChangeDetail={onChangeDetail}
    />
  );
}

describe("VisualPreviewToolbar order", () => {
  it("reads as session, then how the surface is drawn, then what is drawn over it, then what you set", () => {
    const { getByTestId } = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={jest.fn()}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
        onBrowse={jest.fn()}
      />
    );
    const labels: Array<Nullable<string>> = [...getByTestId("editor-toolbar-actions").querySelectorAll("button")].map(
      (it: Element) => it.getAttribute("aria-label")
    );

    // A value picker goes last in every toolbar of this application, so a row always reads as things you flip and
    // then the thing you set. Mesh detail led this row until the rule was settled.
    expect(labels).toEqual([
      "Browse folder",
      "Wireframe",
      "Uv checkerboard",
      "Alpha",
      "Bump",
      "Skeleton",
      "Grid",
      "Axes",
      "Mesh detail",
      "Lighting",
    ]);
  });

  it("rules the groups off from each other, and drops the session rule with the session control", () => {
    const browsing: RenderResult = renderToolbar(0, true);

    // Surface from overlays, overlays from the picker. No session zone here: a session already browsing has nothing
    // to promote, so `onBrowse` is absent and its rule would divide nothing from nothing.
    expect(
      browsing.getByTestId("editor-toolbar-actions").querySelectorAll('[data-testid="editor-toolbar-separator"]')
    ).toHaveLength(2);

    browsing.unmount();

    const single: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={jest.fn()}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
        onBrowse={jest.fn()}
      />
    );

    expect(
      single.getByTestId("editor-toolbar-actions").querySelectorAll('[data-testid="editor-toolbar-separator"]')
    ).toHaveLength(3);
  });
});

describe("VisualPreviewToolbar skeleton toggle", () => {
  it("offers nothing to draw on a model with no bind pose", () => {
    // Every model measured in gamedata carries one, so this state cannot be reached by opening a real file there -
    // which is exactly why it needs a test rather than a look.
    const render: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels
        hasSkeleton={false}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={jest.fn()}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    expect(render.getByRole("button", { name: "Skeleton" })).toBeDisabled();
  });

  it("asks for the overlay when a bind pose is there to draw", () => {
    const changes: Array<IVisualPreviewViewOptions> = [];
    const render: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels={true}
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={(options: IVisualPreviewViewOptions) => changes.push(options)}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    fireEvent.click(render.getByRole("button", { name: "Skeleton" }));

    expect(changes).toHaveLength(1);
    expect(changes[0].isSkeletonVisible).toBe(true);
  });
});

describe("VisualPreviewToolbar bump toggle", () => {
  it("offers nothing to shade on a model whose materials bound no pair", () => {
    // Most models: no descriptor declares a bump, so the toggle sits disabled with its reason rather than vanishing.
    const { getByRole } = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels={true}
        hasSkeleton={true}
        hasBump={false}
        hasAlpha={true}
        onChangeOptions={jest.fn()}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    expect(getByRole("button", { name: "Bump" })).toBeDisabled();
  });

  it("starts on and asks to draw the surface flat when a pair is there to compare against", () => {
    const changes: Array<IVisualPreviewViewOptions> = [];
    const { getByRole } = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels={true}
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={(options: IVisualPreviewViewOptions) => changes.push(options)}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    expect(DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS.isBumpVisible).toBe(true);

    fireEvent.click(getByRole("button", { name: "Bump" }));

    expect(changes).toHaveLength(1);
    expect(changes[0].isBumpVisible).toBe(false);
  });
});

describe("VisualPreviewToolbar alpha toggle", () => {
  it("offers nothing to compare on a model whose every surface is opaque", () => {
    const render: RenderResult = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels={true}
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={false}
        onChangeOptions={jest.fn()}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    expect(render.getByRole("button", { name: "Alpha" })).toBeDisabled();
  });

  it("starts on and asks to draw the surfaces solid for comparison", () => {
    // Solid is what makes a hole in the alpha channel tellable from a hole in the mesh.
    const changes: Array<IVisualPreviewViewOptions> = [];
    const { getByRole } = renderWithProviders(
      <VisualPreviewToolbar
        options={DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS}
        lighting={DEFAULT_VISUAL_LIGHTING}
        detail={0}
        hasDetailLevels={true}
        hasSkeleton={true}
        hasBump={true}
        hasAlpha={true}
        onChangeOptions={(options: IVisualPreviewViewOptions) => changes.push(options)}
        onChangeLighting={jest.fn()}
        onChangeDetail={jest.fn()}
      />
    );

    expect(DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS.isAlphaVisible).toBe(true);

    fireEvent.click(getByRole("button", { name: "Alpha" }));

    expect(changes).toHaveLength(1);
    expect(changes[0].isAlphaVisible).toBe(false);
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
      document.querySelector<HTMLInputElement>(`.${popoverClasses.root} input[type="range"]`)
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
      document.querySelector<HTMLInputElement>(`.${popoverClasses.root} input[type="range"]`)
    );

    // Coarsest stored detail is zero quality, so the handle sits at the left.
    expect(slider!.value).toBe("0");
  });
});
