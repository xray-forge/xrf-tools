import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualBoneVisibility } from "@/core/visuals/components/panels/VisualBonesPanel/VisualBoneVisibility";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockVisualBone,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

async function renderVisibility(
  names: Array<string> = ["wpn_body", "wpn_scope", "wpn_silencer", "magazin"]
): Promise<{ render: RenderResult; service: VisualsService }> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const selected: SelectedVisualDescription = mockSelectedVisual({
    description: mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer)],
      bufferLength: buffer.byteLength,
      bones: names.map((name: string) => mockVisualBone({ name, parent: name === "wpn_body" ? "" : "wpn_body" })),
    }),
  });

  setMockInvokeResponses({
    ["plugin:visuals|open_model"]: selected,
    ["plugin:visuals|read_geometry"]: buffer.toArrayBuffer(),
  });

  const container: Container = mockContainer([
    VisualLoadService,
    VisualMotionService,
    VisualsService,
    // The binding an application makes at its composition root, which is what the panel resolves through.
    { token: VISUAL_INSPECTION, factory: (it: Container) => it.get(VisualsService) },
  ]);
  const service: VisualsService = container.get(VisualsService);

  await service.openFile("C:\\gamedata\\wpn_ak74.ogf");

  return { render: renderWithProviders(<VisualBoneVisibility />, { container }), service };
}

describe("VisualBoneVisibility", () => {
  it("offers a control for each addon bone the visual carries, and nothing for the rest", async () => {
    const { render } = await renderVisibility();

    expect(render.getByLabelText("Show wpn_scope")).toBeInTheDocument();
    expect(render.getByLabelText("Show wpn_silencer")).toBeInTheDocument();
    expect(render.queryByLabelText("Show magazin")).toBeNull();
  });

  it("turns an addon off, which is the state the game shows when it is not attached", async () => {
    const { render, service } = await renderVisibility();

    fireEvent.click(render.getByLabelText("Show wpn_scope"));

    expect(service.hiddenBones.has("wpn_scope")).toBe(true);
    expect(render.getByLabelText("Show wpn_scope")).not.toBeChecked();
  });

  it("hides any other bone through the selection the tree made", async () => {
    const { render, service } = await renderVisibility();

    // Through `act`, because the tree that normally makes this selection is not rendered here.
    act(() => service.highlightBone("magazin"));

    fireEvent.click(render.getByRole("button", { name: "Hide magazin" }));

    expect(service.hiddenBones.has("magazin")).toBe(true);
    // Listed as a chip because it has no control of its own, unlike the addon bones above it.
    expect(render.getByRole("button", { name: "magazin" })).toBeInTheDocument();
  });

  it("says how to hide a bone while none is selected", async () => {
    const { render } = await renderVisibility();

    expect(render.getByText(/Pick a bone above to hide it/)).toBeInTheDocument();
  });

  it("brings everything back at once", async () => {
    const { render, service } = await renderVisibility();

    fireEvent.click(render.getByLabelText("Show wpn_scope"));
    fireEvent.click(render.getByRole("button", { name: "Show all" }));

    expect(service.hiddenBones.size).toBe(0);
  });

  it("warns when a skeleton reaches past the engine's visibility mask", async () => {
    const many: Array<string> = Array.from({ length: 70 }, (_, index: number) => `bone_${index}`);

    const { render } = await renderVisibility(many);

    expect(render.getByText(/6 of these bones sit past the engine's 64 bone visibility mask/)).toBeInTheDocument();
  });
});
