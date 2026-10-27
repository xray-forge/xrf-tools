import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { RenderGraphSettings } from "@xrf/ipc/types/xrf-renderer";

import {
  DEFAULT_RENDER_GRAPH_SETTINGS,
  SERIAL_RENDER_GRAPH_SETTINGS,
} from "@/core/render/lib/settings/render-graph-settings";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelRendererGraphSection } from "@/tools/level-viewer/components/panels/LevelRendererPanel/LevelRendererGraphSection";

function renderSection(settings: RenderGraphSettings, onChange: (settings: RenderGraphSettings) => void) {
  return renderWithProviders(<LevelRendererGraphSection graph={null} settings={settings} onChange={onChange} />);
}

describe("LevelRendererGraphSection", () => {
  it("turns the frame graph's optimizations off one at a time, to bisect a capture", async () => {
    const onChange = jest.fn<(settings: RenderGraphSettings) => void>();
    const { getByRole } = renderSection(DEFAULT_RENDER_GRAPH_SETTINGS, onChange);

    await userEvent.click(getByRole("checkbox", { name: "Merge render passes" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...DEFAULT_RENDER_GRAPH_SETTINGS, isMerging: false });
  });

  // With any off, the switch for them all turns them back on first.
  it("turns them all off at once, or all back on once any is off", async () => {
    const onChange = jest.fn<(settings: RenderGraphSettings) => void>();

    const whole = renderSection(DEFAULT_RENDER_GRAPH_SETTINGS, onChange);

    await userEvent.click(whole.getByRole("checkbox", { name: "Optimizations" }));

    expect(onChange).toHaveBeenLastCalledWith(SERIAL_RENDER_GRAPH_SETTINGS);

    whole.unmount();

    const bisected = renderSection({ ...DEFAULT_RENDER_GRAPH_SETTINGS, isMerging: false }, onChange);

    await userEvent.click(bisected.getByRole("checkbox", { name: "Optimizations" }));

    expect(onChange).toHaveBeenLastCalledWith(DEFAULT_RENDER_GRAPH_SETTINGS);
  });
});
