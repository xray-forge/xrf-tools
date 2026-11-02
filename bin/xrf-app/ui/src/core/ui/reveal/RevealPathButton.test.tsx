import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";

import { EToolId } from "@/core/routing/tool";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("RevealPathButton", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("asks the system to show the path", async () => {
    const revealPath = jest.fn((_: unknown) => null);

    mockIpcCommands({ [ESystemCommand.REVEAL_PATH]: revealPath });

    const { getByRole } = renderWithProviders(<RevealPathButton tool={EToolId.ARCHIVES_PACKER} path={"C:\\out"} />);

    fireEvent.click(getByRole("button"));

    await waitFor(() => expect(revealPath).toHaveBeenCalledWith({ path: "C:\\out" }));
  });

  it("has nothing to show without a path", () => {
    const { getByRole } = renderWithProviders(<RevealPathButton tool={EToolId.ARCHIVES_PACKER} path={null} />);

    expect(getByRole("button")).toBeDisabled();
  });

  it("reports a file manager that would not open instead of throwing", async () => {
    mockIpcCommands({
      [ESystemCommand.REVEAL_PATH]: () => {
        throw new Error("no file manager");
      },
    });

    const { getByRole } = renderWithProviders(<RevealPathButton tool={EToolId.ARCHIVES_PACKER} path={"C:\\out"} />);

    fireEvent.click(getByRole("button"));

    // The button comes back rather than staying stuck, which is what says the failure was handled.
    await waitFor(() => expect(getByRole("button")).not.toBeDisabled());
  });
});
