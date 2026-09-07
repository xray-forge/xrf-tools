import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { runInAction } from "@wirestate/mobx";

import { ESequenceMotionState, VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { mockVisualMotionBake } from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SequenceTrackPanel } from "./SequenceTrackPanel";

/** A track with baked motions, ready for row actions without a backend read. */
function renderTrack(names: ReadonlyArray<string> = ["first", "second"]) {
  const { container, service } = mockInjectedService(VisualSequenceService);

  runInAction(() => {
    service.clips = names.map((motion, index) => ({ id: `clip-${index + 1}`, motion }));
    service.motions = new Map(
      names.map((name) => [
        name,
        {
          state: ESequenceMotionState.READY,
          reason: null,
          bake: mockVisualMotionBake({ name, frameCount: 4 }),
          transforms: null,
        },
      ])
    );
  });

  service.seek(1, 2);

  return { service, view: renderWithProviders(<SequenceTrackPanel />, { container }) };
}

describe("SequenceTrackPanel", () => {
  it.each(["{Enter}", " "])("seeks to the start of a clip with %s", async (key) => {
    const { service, view } = renderTrack();

    expect(service.clip?.id).toBe("clip-2");
    expect(service.frame).toBe(2);

    await userEvent.tab();

    expect(view.getByRole("button", { name: "Seek to first" })).toHaveFocus();

    await userEvent.keyboard(key);

    expect(service.clip?.id).toBe("clip-1");
    expect(service.frame).toBe(0);
  });

  it("updates movement boundaries while keeping the current clip and frame", async () => {
    const { service, view } = renderTrack();
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
    const { service, view } = renderTrack(["idle", "idle"]);
    const remove = view.getAllByRole("button", { name: "Remove idle" });

    await userEvent.click(remove[1]);

    expect(service.clips.map((clip) => clip.id)).toEqual(["clip-1"]);
    expect(view.getAllByRole("button", { name: "Seek to idle" })).toHaveLength(1);
    expect(view.getByRole("button", { name: "Move idle earlier" })).toBeDisabled();
    expect(view.getByRole("button", { name: "Move idle later" })).toBeDisabled();
  });
});
