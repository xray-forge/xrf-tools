import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockSelectedVisual,
  mockVisualModelViews,
  mockVisualMotionBake,
  mockVisualMotionTransforms,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AsyncState } from "@/lib/async-state";

import { SequenceTrackPanel } from "./SequenceTrackPanel";

async function renderTrack(names: ReadonlyArray<string> = ["first", "second"]) {
  const { container, service } = mockInjectedService(VisualSequenceService, [VisualLoadService]);
  const bake = mockVisualMotionBake({ frameCount: 4 });

  container.get(VisualLoadService).visual = AsyncState.ready({
    selected: mockSessionSnapshot(mockSelectedVisual()),
    views: mockVisualModelViews(),
  });

  setMockInvokeResponses({
    ["plugin:visuals|open_motion"]: mockSessionResponse(bake),
    ["plugin:visuals|read_motion"]: mockVisualMotionTransforms(bake),
  });

  for (const name of names) {
    service.add(name);
  }

  await waitFor(() => expect(service.playableCount).toBe(names.length));

  service.seek(1, 2);

  return { service, view: renderWithProviders(<SequenceTrackPanel />, { container }) };
}

describe("SequenceTrackPanel", () => {
  afterEach(() => {
    resetMockInvoke();
  });

  it.each(["{Enter}", " "])("seeks to the start of a clip with %s", async (key) => {
    const { service, view } = await renderTrack();

    expect(service.clip?.id).toBe("clip-2");
    expect(service.frame).toBe(2);

    await userEvent.tab();

    expect(view.getByRole("button", { name: "Seek to first" })).toHaveFocus();

    await userEvent.keyboard(key);

    expect(service.clip?.id).toBe("clip-1");
    expect(service.frame).toBe(0);
  });

  it("updates movement boundaries while keeping the current clip and frame", async () => {
    const { service, view } = await renderTrack();
    const earlier = view.getByRole("button", { name: "Move first earlier" });
    const later = view.getByRole("button", { name: "Move second later" });

    expect(earlier).toBeDisabled();
    expect(earlier).toHaveAccessibleDescription("Already first in the track");
    expect(later).toBeDisabled();
    expect(later).toHaveAccessibleDescription("Already last in the track");

    await userEvent.click(view.getByRole("button", { name: "Move first later" }));

    expect(service.clips.map((clip) => clip.id)).toEqual(["clip-2", "clip-1"]);
    expect(service.clip?.id).toBe("clip-2");
    expect(service.frame).toBe(2);
    expect(view.getByRole("button", { name: "Move first later" })).toBeDisabled();
    expect(earlier).toBeEnabled();

    await userEvent.click(earlier);

    expect(service.clips.map((clip) => clip.id)).toEqual(["clip-1", "clip-2"]);
    expect(earlier).toBeDisabled();
    expect(later).toBeDisabled();
  });

  it("removes the chosen occurrence when a motion appears twice", async () => {
    const { service, view } = await renderTrack(["idle", "idle"]);
    const remove = view.getAllByRole("button", { name: "Remove idle" });

    await userEvent.click(remove[1]);

    expect(service.clips.map((clip) => clip.id)).toEqual(["clip-1"]);
    expect(view.getAllByRole("button", { name: "Seek to idle" })).toHaveLength(1);
    expect(view.getByRole("button", { name: "Move idle earlier" })).toBeDisabled();
    expect(view.getByRole("button", { name: "Move idle later" })).toBeDisabled();
  });
});
