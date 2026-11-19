import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor, within } from "@testing-library/react";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { mockIpcCommands } from "@xrf/ipc/testing";

import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { clearLocalStorage, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SpawnPackerTool } from "./SpawnPackerTool";

describe("SpawnPackerTool outcomes", () => {
  beforeEach(() => {
    clearLocalStorage();
    setLocalStorageValue("xrf.form.spawn-packer.source", "C:\\source");
    setLocalStorageValue("xrf.form.spawn-packer.destination", "C:\\destination");
  });

  it("asks before replacing an output already there, and packs again forced once agreed", async () => {
    const forced: Array<boolean> = [];

    mockIpcCommands({
      [ESpawnCommand.PACK_FILE]: ({ request }) => {
        forced.push(request.isForced);

        if (!request.isForced) {
          throw new Error("C:\\ already holds what this run writes (destination); force it to replace them");
        }

        return { operation: "pack", destination: "C:\\destination", outcome: "completed" };
      },
    });

    const view = renderWithProviders(<SpawnPackerTool />, {
      route: "/spawn-packer",
      bindings: [SpawnConversionService],
    });
    const submit = await view.findByRole("button", { name: "Pack" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    const dialog: HTMLElement = await view.findByRole("dialog", { name: "Replace existing files?" });

    expect(dialog).toHaveTextContent("already holds what this run writes (destination)");

    fireEvent.click(within(dialog).getByRole("button", { name: "Replace" }));

    expect(await view.findByText(/Packed spawn/)).toBeInTheDocument();
    expect(forced).toEqual([false, true]);
  });

  it("replaces nothing where the question is declined", async () => {
    const forced: Array<boolean> = [];

    mockIpcCommands({
      [ESpawnCommand.PACK_FILE]: ({ request }) => {
        forced.push(request.isForced);

        throw new Error("C:\\ already holds what this run writes (destination); force it to replace them");
      },
    });

    const view = renderWithProviders(<SpawnPackerTool />, {
      route: "/spawn-packer",
      bindings: [SpawnConversionService],
    });
    const submit = await view.findByRole("button", { name: "Pack" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);
    fireEvent.click(
      within(await view.findByRole("dialog", { name: "Replace existing files?" })).getByRole("button", {
        name: "Cancel",
      })
    );

    await waitFor(() => expect(view.queryByRole("dialog")).not.toBeInTheDocument());
    expect(forced).toEqual([false]);
  });

  describe.each(["Source", "Output spawn"])("editing %s", (label) => {
    it.each(["success", "failure"])("clears the previous %s", async (outcome) => {
      mockIpcCommands({
        [ESpawnCommand.PACK_FILE]: () => {
          if (outcome === "failure") {
            throw new Error("Cannot process spawn");
          }

          return { operation: "pack", destination: "C:\\destination", outcome: "completed" };
        },
      });

      const view = renderWithProviders(<SpawnPackerTool />, {
        route: "/spawn-packer",
        bindings: [SpawnConversionService],
      });
      const submit = await view.findByRole("button", { name: "Pack" });

      await waitFor(() => expect(submit).toBeEnabled());
      fireEvent.click(submit);

      const message = outcome === "success" ? /Packed spawn/ : /Cannot process spawn/;

      expect(await view.findByText(message)).toBeInTheDocument();

      if (outcome === "success") {
        fireEvent.click(view.getByRole("button", { name: "Show parameters" }));
      }

      fireEvent.change(view.getByRole("textbox", { name: label }), { target: { value: "C:\\changed" } });

      expect(view.queryByText(message)).not.toBeInTheDocument();
    });
  });
});
