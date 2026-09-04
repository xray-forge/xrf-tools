import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { VisualMotionsPanel } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionsPanel";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { VisualMotionBake, VisualMotionDependency } from "@/core/bindings/types/xrf-visual";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  MockVisualBuffer,
  mockVisualDescription,
  mockVisualMotionBake,
  mockVisualMotionTransforms,
} from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** One resolved omf reference, which is what makes a visual animated as far as the panel is concerned. */
function mockMotionRef(reference: string = "actors\\stalker_animation"): VisualMotionDependency {
  return {
    reference,
    resolution: {
      kind: "resolved",
      step: "installation",
      assets: [{ container: { kind: "archive", path: "C:\\game\\db" }, logicalPath: `meshes\\${reference}.omf` }],
    },
  };
}

/**
 * Opens a visual through the real service, then renders the panel over it.
 *
 * @param motions - Motion references the opened visual carries; none makes it a static model.
 * @param names - What `list_motions` answers for it.
 * @returns The render result and the container the panel resolves through.
 */
async function renderPanel(
  motions: Array<VisualMotionDependency>,
  names: Array<string> = []
): Promise<{ render: RenderResult; container: Container; listCalls: () => number }> {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const selected: SelectedVisualDescription = mockSelectedVisual({
    dependencies: { motions, textures: [] },
    description: mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer)],
      bufferLength: buffer.byteLength,
      embeddedMotions: [],
    }),
  });

  // One bake behind both motion reads: the loader refuses bytes whose length disagrees with the counts it was told.
  const bake: VisualMotionBake = mockVisualMotionBake({ frameCount: 4, boneCount: 1 });

  let listed: number = 0;

  setMockInvokeResponses({
    ["plugin:visuals|open_model"]: selected,
    ["plugin:visuals|read_geometry"]: buffer.toArrayBuffer(),
    ["plugin:visuals|list_motions"]: () => {
      listed += 1;

      return names;
    },
    ["plugin:visuals|open_motion"]: (parameters?: Record<string, unknown>) => ({
      ...bake,
      name: String((parameters as { name: string }).name),
    }),
    ["plugin:visuals|read_motion"]: mockVisualMotionTransforms(bake),
  });

  const container: Container = mockContainer([VisualLoadService, VisualMotionService, VisualsService]);

  await container.get(VisualsService).openFile("C:\\gamedata\\meshes\\stalker.ogf");

  const render: RenderResult = renderWithProviders(<VisualMotionsPanel />, { container });

  if (motions.length) {
    await act(async () => await container.get(VisualMotionService).list());
  }

  return { container, listCalls: () => listed, render };
}

describe("VisualMotionsPanel", () => {
  it("gathers what the visual plays into families, and offers no transport before a pose", async () => {
    // The families are collapsed, which is the point of them: a stalker names about 2,500 motions and a wall of them
    // is only readable by someone who already knows what to type.
    const { render } = await renderPanel([mockMotionRef()], ["stand_idle_0", "stand_idle_1"]);

    expect(await render.findByText("stand (2)")).toBeInTheDocument();
    expect(render.queryByText("stand_idle_0")).toBeNull();

    fireEvent.dblClick(render.getByText("stand (2)"));

    expect(await render.findByText("stand_idle_0")).toBeInTheDocument();
    expect(render.getByText("stand_idle_1")).toBeInTheDocument();

    // Nothing is posed yet, so there are no frames to play or scrub.
    expect(render.getByRole("button", { name: "Play" })).toBeDisabled();
    expect(render.getByText("0 / 0")).toBeInTheDocument();
  });

  it("reads no animation file for a model that references none", async () => {
    // The guard the sequencer's listing already applies: naming motions means reading every referenced omf, and a
    // static model has nothing for that read to find.
    const { render, listCalls } = await renderPanel([]);

    expect(render.getByText("No motions. Resolved from the visual's omf motion refs.")).toBeInTheDocument();
    expect(render.queryByRole("textbox", { name: "Filter motions" })).toBeNull();
    expect(listCalls()).toBe(0);
  });

  it("opens what a filter matched, because a closed family reads as no answer", async () => {
    const { render } = await renderPanel(
      [mockMotionRef()],
      ["stand_idle_0", "stand_idle_1", "crouch_walk_fwd", "crouch_walk_back"]
    );

    await render.findByText("stand (2)");

    fireEvent.change(render.getByRole("textbox", { name: "Filter motions" }), { target: { value: "walk_fwd" } });

    expect(await render.findByText("crouch_walk_fwd")).toBeInTheDocument();
    expect(render.queryByText("crouch_walk_back")).toBeNull();
    expect(render.queryByText("stand (2)")).toBeNull();
  });

  it("says how much it searched when a filter matches nothing", async () => {
    const { render } = await renderPanel([mockMotionRef()], ["stand_idle_0", "crouch_walk_fwd"]);

    await render.findByText("stand_idle_0");

    fireEvent.change(render.getByRole("textbox", { name: "Filter motions" }), { target: { value: "swim" } });

    expect(await render.findByText("No motion of the 2 this visual plays matches that.")).toBeInTheDocument();
  });

  it("poses on activation rather than on selection, since a bake is work", async () => {
    const { render, container } = await renderPanel([mockMotionRef()], ["stand_idle_0"]);

    const row: HTMLElement = await render.findByText("stand_idle_0");

    // Arrowing through 2,500 names would otherwise read and bake one motion per keystroke.
    fireEvent.click(row);

    expect(container.get(VisualMotionService).posed.value).toBeNull();

    fireEvent.dblClick(row);

    await waitFor(() => expect(container.get(VisualMotionService).frameCount).toBe(4));

    expect(render.getByText(/\d+ \/ 4/)).toBeInTheDocument();
    expect(render.getByRole("button", { name: "Pause" })).toBeEnabled();
  });

  it("names the omf files the engine loads beside what it plays", async () => {
    const { render } = await renderPanel([mockMotionRef("actors\\stalker_animation")], ["stand_idle_0"]);

    expect(await render.findByText("Motion refs (1)")).toBeInTheDocument();
    expect(render.getByText("actors\\stalker_animation")).toBeInTheDocument();
  });
});

describe("VisualMotionsPanel listing", () => {
  it("says it is reading, rather than claiming the visual plays nothing", async () => {
    resetMockInvoke();

    const buffer: MockVisualBuffer = new MockVisualBuffer();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSelectedVisual({
        dependencies: { motions: [mockMotionRef()], textures: [] },
        description: mockVisualDescription({
          submeshes: [mockPackedSubmesh(buffer)],
          bufferLength: buffer.byteLength,
        }),
      }),
      ["plugin:visuals|read_geometry"]: buffer.toArrayBuffer(),
      // Never settles, which is the state a 2,500-name actor is in for as long as its omf files are being read.
      ["plugin:visuals|list_motions"]: () => new Promise<Array<string>>(() => {}),
    });

    const container: Container = mockContainer([VisualLoadService, VisualMotionService, VisualsService]);

    await container.get(VisualsService).openFile("C:\\gamedata\\meshes\\stalker.ogf");

    const render: RenderResult = renderWithProviders(<VisualMotionsPanel />, { container });

    expect(
      await render.findByText("Listing motions. Every animation file the visual references is read once.")
    ).toBeInTheDocument();
  });
});
