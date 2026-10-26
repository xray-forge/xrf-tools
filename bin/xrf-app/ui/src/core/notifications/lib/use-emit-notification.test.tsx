import { describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ENotificationSeverity,
  INotification,
  TEmitNotification,
  useEmitNotification,
} from "@/core/notifications/lib/index";
import { NotificationsService } from "@/core/notifications/services";
import { EApplicationId } from "@/core/routing/application";
import { renderWithProviders } from "@/fixtures/utils/render";

/**
 * Stands in for a page that learns its outcome in the component rather than in a service.
 *
 * It renders what the store holds as well as raising it, so the assertion runs through the whole path
 * the pages use: hook to bus to the `@OnEvent` handler on the root bound service.
 *
 * @returns Test component that emits and displays notifications.
 */
function NotifyingPage(): ReactElement {
  const notify: TEmitNotification = useEmitNotification();
  const notificationsService: NotificationsService = useInjection(NotificationsService);

  return (
    <div>
      <button
        onClick={() =>
          notify({
            details: "C:\\configs",
            severity: ENotificationSeverity.WARNING,
            source: EApplicationId.CONFIGS_VERIFIER,
            title: "Configs did not pass validation",
          })
        }
      >
        Verify
      </button>

      <span data-testid={"recorded"}>
        {notificationsService.notifications
          .map((it: INotification) => `${it.source}/${it.severity}/${it.title}/${it.details}`)
          .join("|")}
      </span>
    </div>
  );
}

describe("useNotify", () => {
  it("delivers a component raised outcome to the notification centre", async () => {
    const { getByRole, getByTestId } = renderWithProviders(<NotifyingPage />);

    expect(getByTestId("recorded")).toHaveTextContent("");

    await userEvent.click(getByRole("button", { name: "Verify" }));

    expect(getByTestId("recorded")).toHaveTextContent(
      "configs-verifier/warning/Configs did not pass validation/C:\\configs"
    );
  });

  it("records every raise rather than collapsing repeats", async () => {
    const { getByRole, getByTestId } = renderWithProviders(<NotifyingPage />);

    await userEvent.click(getByRole("button", { name: "Verify" }));
    await userEvent.click(getByRole("button", { name: "Verify" }));

    expect(getByTestId("recorded").textContent?.split("|")).toHaveLength(2);
  });
});
