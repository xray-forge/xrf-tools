import { describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { NotificationsService } from "@/core/notifications/services";
import { EApplicationId } from "@/core/routing/application";
import { SettingsService } from "@/core/settings/services/settings";
import { NotificationsPanel } from "@/core/shell/panel/notifications/NotificationsPanel";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

interface IPanelRender {
  render: RenderResult;
  service: NotificationsService;
}

/**
 * Renders the notification panel with an optional seeded log.
 *
 * @param seed - Notifications recorded before the panel opens.
 * @param isDevModeEnabled - Whether developer notifications are visible.
 * @returns Render result and the notifications service used by the panel.
 */
function renderPanel(seed: Array<INotificationPayload> = [], isDevModeEnabled: boolean = false): IPanelRender {
  const container: Container = mockContainer();
  const service: NotificationsService = container.get(NotificationsService);
  const settingsService: SettingsService = container.get(SettingsService);

  settingsService.setDevModeEnabled(isDevModeEnabled);
  seed.forEach((payload: INotificationPayload) => service.push(payload));

  return {
    render: renderWithProviders(<NotificationsPanel />, { container }),
    service,
  };
}

describe("NotificationsPanel", () => {
  it("says nothing has happened rather than showing an empty box", () => {
    const { render } = renderPanel();

    expect(render.getByText(/Nothing has been reported yet/)).toBeInTheDocument();
    expect(render.getByRole("button", { name: "Clear all" })).toBeDisabled();
  });

  it("names the tool a record came from the way the rail does", () => {
    const { render } = renderPanel([
      {
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Extracted textures",
      },
    ]);

    expect(render.getByText("Extracted textures")).toBeInTheDocument();
    expect(render.getByText("Archives explorer")).toBeInTheDocument();
  });

  it("shows the newest record first", () => {
    const { render } = renderPanel([
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Older" },
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Newer" },
    ]);

    const titles: Array<string> = render.getAllByText(/Older|Newer/).map((it: HTMLElement) => it.textContent as string);

    expect(titles).toEqual(["Newer", "Older"]);
  });

  it("reads what it was opened onto", () => {
    const { service } = renderPanel([
      { severity: ENotificationSeverity.ERROR, source: EApplicationId.ARCHIVES_EXPLORER, title: "Failed" },
    ]);

    expect(service.unreadCount).toBe(0);
  });

  it("reads what arrives while it is open", () => {
    const { service } = renderPanel();

    act(() =>
      service.push({ severity: ENotificationSeverity.ERROR, source: EApplicationId.ARCHIVES_EXPLORER, title: "Failed" })
    );

    // Left unread, the badge counts records the user is looking at and nothing can dismiss it.
    expect(service.unreadCount).toBe(0);
  });

  it("keeps details out of the way until they are asked for", async () => {
    const { render } = renderPanel([
      {
        details: "Read 4128 entries\nC:\\gamedata\\gamedata.db0",
        severity: ENotificationSeverity.INFO,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Opened archive",
      },
    ]);

    expect(render.queryByText(/gamedata\.db0/)).not.toBeInTheDocument();

    await userEvent.click(render.getByLabelText("Show details"));

    expect(render.getByText(/gamedata\.db0/)).toBeInTheDocument();
  });

  it("states why a failure failed without being asked", async () => {
    const { render } = renderPanel([
      {
        details: "Access denied\nC:\\out\\system.ltx",
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Could not extract",
      },
    ]);

    // The first line of a refusal is the reason for it, which is the one line worth spending unprompted,
    // and the ellipsis says there is more behind it.
    expect(render.getByText("Access denied …")).toBeInTheDocument();
    expect(render.queryByText(/system\.ltx/)).not.toBeInTheDocument();

    await userEvent.click(render.getByLabelText("Show details"));

    expect(render.getByText(/system\.ltx/)).toBeInTheDocument();
  });

  it("cuts a long body off with an ellipsis until the rest is asked for", async () => {
    const lines: Array<string> = Array.from({ length: 12 }, (_, index: number) => `line ${index + 1}`);
    const { render } = renderPanel([
      {
        details: lines.join("\n"),
        severity: ENotificationSeverity.INFO,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Read archive",
      },
    ]);

    await userEvent.click(render.getByLabelText("Show details"));

    // A body that fills the panel buries the records around it, so it stops and says that it stopped.
    expect(render.getByText(/…/)).toBeInTheDocument();
    expect(render.queryByText(/line 9/)).not.toBeInTheDocument();

    await userEvent.click(render.getByRole("button", { name: "Show all 12 lines" }));

    expect(render.getByText(/line 12/)).toBeInTheDocument();
  });

  it("spends no line previewing a routine outcome", () => {
    const { render } = renderPanel([
      {
        details: "Wrote 214 files",
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Extracted textures",
      },
    ]);

    expect(render.queryByText("Wrote 214 files")).not.toBeInTheDocument();
  });

  it("keeps one record open at a time", async () => {
    const { render } = renderPanel([
      {
        details: "Older body",
        severity: ENotificationSeverity.INFO,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Older",
      },
      {
        details: "Newer body",
        severity: ENotificationSeverity.INFO,
        source: EApplicationId.ARCHIVES_EXPLORER,
        title: "Newer",
      },
    ]);

    await userEvent.click(render.getAllByLabelText("Show details")[0]);

    expect(render.getByText("Newer body")).toBeInTheDocument();

    await userEvent.click(render.getAllByLabelText("Show details")[0]);

    // Two open traces in a panel this narrow leave no room for the outcomes around them.
    expect(render.queryByText("Newer body")).not.toBeInTheDocument();
    expect(render.getByText("Older body")).toBeInTheDocument();
  });

  it("collapses a repeated outcome into one counted row", () => {
    const { render } = renderPanel([
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Rebuilt index" },
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Rebuilt index" },
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Rebuilt index" },
    ]);

    expect(render.getAllByText("Rebuilt index")).toHaveLength(1);
    expect(render.getByText("×3")).toBeInTheDocument();
  });

  it("names the tool on every record, however long the run from it", () => {
    const { render } = renderPanel([
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Older" },
      { severity: ENotificationSeverity.SUCCESS, source: EApplicationId.ARCHIVES_EXPLORER, title: "Newer" },
    ]);

    // Which tool reported an outcome is never the thing to drop: a log that hides it reads as one tool.
    expect(render.getAllByText("Archives explorer")).toHaveLength(2);
  });

  it("offers no expander for a record with nothing more to say", () => {
    const { render } = renderPanel([
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Nothing to expand" },
    ]);

    expect(render.queryByLabelText("Show details")).not.toBeInTheDocument();
  });

  it("hides dev traces while dev mode is off", () => {
    const { render } = renderPanel([
      { severity: ENotificationSeverity.DEV, source: EApplicationId.SPRITE_EQUIPMENT_EDITOR, title: "grid recomputed" },
      {
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
        title: "Packed sprite",
      },
    ]);

    expect(render.getByText("Packed sprite")).toBeInTheDocument();
    expect(render.queryByText("grid recomputed")).not.toBeInTheDocument();
  });

  it("reveals traces that were recorded before dev mode was turned on", () => {
    const { render } = renderPanel(
      [
        {
          severity: ENotificationSeverity.DEV,
          source: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
          title: "grid recomputed",
        },
      ],
      true
    );

    // The point of recording them regardless: the switch is useful after something odd happened, not
    // only before it.
    expect(render.getByText("grid recomputed")).toBeInTheDocument();
    // A trace states what it is through the terminal icon and the monospace face, not a badge that
    // would cost a line of its own in a panel that is mostly traces once the switch is on.
    expect(render.getByTestId("TerminalIcon")).toBeInTheDocument();
  });

  it("does not mark a real outcome as a dev trace", () => {
    const { render } = renderPanel(
      [
        {
          severity: ENotificationSeverity.SUCCESS,
          source: EApplicationId.SPRITE_EQUIPMENT_EDITOR,
          title: "Packed sprite",
        },
      ],
      true
    );

    expect(render.queryByTestId("TerminalIcon")).not.toBeInTheDocument();
  });

  it("clears the log on request", async () => {
    const { render, service } = renderPanel([
      { severity: ENotificationSeverity.INFO, source: EApplicationId.ARCHIVES_EXPLORER, title: "Something" },
    ]);

    await userEvent.click(render.getByRole("button", { name: "Clear all" }));

    expect(service.notifications).toHaveLength(0);
    expect(render.getByText(/Nothing has been reported yet/)).toBeInTheDocument();
  });
});
