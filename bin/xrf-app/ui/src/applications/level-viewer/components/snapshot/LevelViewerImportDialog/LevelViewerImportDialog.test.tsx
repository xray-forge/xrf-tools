import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelViewerImportDialog } from "@/applications/level-viewer/components/snapshot/LevelViewerImportDialog";
import {
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
} from "@/applications/level-viewer/lib/snapshot/level-viewer-snapshot";
import { renderWithProviders } from "@/fixtures/utils/render";

const SNAPSHOT: ILevelViewerSnapshot = {
  build: { commit: "0123456789abcdef", isDirty: false } as ILevelViewerSnapshot["build"],
  exportedAt: "2026-10-20T16:30:00.000Z",
  format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
  level: "levels/zaton",
};

const ENTRIES: Array<ILevelViewerSnapshotEntry> = [
  entry("camera.x", ELevelViewerSnapshotStatus.SKIP, 5, 1, { applied: undefined, reason: "taken on levels/zaton" }),
  entry("camera.fieldOfView", ELevelViewerSnapshotStatus.CHANGE, 60, 80),
  entry("weather.time", ELevelViewerSnapshotStatus.CHANGE, 25_200, 60_480),
  entry("settings.reflections.steps", ELevelViewerSnapshotStatus.CHANGE, 64, 48),
  entry("settings.reflections.traceInterval", ELevelViewerSnapshotStatus.CHANGE, 12, 2, { applied: 8 }),
  entry("settings.reflections.quality", ELevelViewerSnapshotStatus.SKIP, "medium", undefined, {
    applied: undefined,
    reason: "not a setting the view sets for itself",
  }),
  entry("settings.shadows.swayRate", ELevelViewerSnapshotStatus.SAME, 60, 60),
  entry("options.isGrassy", ELevelViewerSnapshotStatus.SAME, true, true),
];

function entry(
  key: string,
  status: ELevelViewerSnapshotStatus,
  value: unknown,
  current: unknown,
  overrides: Partial<ILevelViewerSnapshotEntry> = {}
): ILevelViewerSnapshotEntry {
  return {
    applied: value,
    current,
    group: key.startsWith("settings.") ? key.split(".").slice(0, 2).join(".") : key.split(".")[0],
    key,
    status,
    value,
    ...overrides,
  } as ILevelViewerSnapshotEntry;
}

function renderDialog(level: string = "levels/zaton"): RenderResult & { onApply: jest.Mock } {
  const onApply: jest.Mock = jest.fn();

  return {
    ...renderWithProviders(
      <LevelViewerImportDialog
        fileName={"zaton.json"}
        snapshot={SNAPSHOT}
        entries={ENTRIES}
        level={level}
        result={null}
        onApply={onApply}
        onClose={() => undefined}
      />
    ),
    onApply,
  };
}

describe("LevelViewerImportDialog", () => {
  it("offers every change chosen, by part, and lists what it does not apply apart", () => {
    const view: RenderResult = renderDialog();

    expect(view.getByText("4 of 4 chosen")).toBeInTheDocument();
    expect(view.getByRole("checkbox", { name: "Apply settings.reflections.steps" })).toBeChecked();
    expect(view.getByRole("checkbox", { name: "Apply settings.reflections.traceInterval" })).toBeChecked();
    expect(view.getByText("file 12")).toBeInTheDocument();
    expect(view.getByText("07:00")).toBeInTheDocument();
    expect(view.getByText("not a setting the view sets for itself")).toBeInTheDocument();
    expect(view.queryByRole("checkbox", { name: "Apply settings.reflections.quality" })).not.toBeInTheDocument();
    expect(view.queryByRole("checkbox", { name: "Apply camera.x" })).not.toBeInTheDocument();
    expect(view.getByRole("button", { name: "Apply 4" })).toBeEnabled();
  });

  it("applies only what stays chosen, a part at a time or a value at a time", async () => {
    const view: RenderResult & { onApply: jest.Mock } = renderDialog();

    await userEvent.click(view.getByRole("checkbox", { name: "Apply Reflections" }));
    await userEvent.click(view.getByRole("checkbox", { name: "Apply settings.reflections.steps" }));
    await userEvent.click(view.getByRole("button", { name: "Apply 3" }));

    expect(
      (view.onApply.mock.calls[0][0] as Array<ILevelViewerSnapshotEntry>).map((it: ILevelViewerSnapshotEntry) => it.key)
    ).toEqual(["camera.fieldOfView", "weather.time", "settings.reflections.steps"]);
  });

  it("shows what stays as it is on asking", async () => {
    const view: RenderResult = renderDialog();
    const unchanged: HTMLElement = view.getByText("Unchanged").closest("[data-testid=detail-section]") as HTMLElement;

    expect(within(unchanged).queryByText("swayRate")).not.toBeInTheDocument();

    await userEvent.click(within(unchanged).getByRole("button", { name: "Show" }));

    expect(within(unchanged).getByText("swayRate")).toBeInTheDocument();
    expect(within(unchanged).getByText("isGrassy")).toBeInTheDocument();
  });

  it("warns that a view of another level leaves the camera where it is", () => {
    const view: RenderResult = renderDialog("levels/jupiter");

    expect(view.getByText(/Taken on levels\/zaton while levels\/jupiter is open/)).toBeInTheDocument();
  });
});
