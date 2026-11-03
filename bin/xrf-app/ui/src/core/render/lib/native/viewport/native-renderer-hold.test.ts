import { beforeEach, describe, expect, it } from "@jest/globals";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { Logger } from "@xrf/logging";

import { NativeRendererHold } from "@/core/render/lib/native/viewport/native-renderer-hold";
import { muteConsole } from "@/fixtures/utils/console";
import { deferred, flush, IDeferred } from "@/fixtures/utils/promise";

function createHold(): NativeRendererHold {
  return new NativeRendererHold(new Logger("native-renderer-hold.test"));
}

describe("NativeRendererHold", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("holds once however often taken, and releases the hold the renderer numbered", async () => {
    mockIpcCommands({ [ERenderCommand.HOLD]: 4 });

    const hold: NativeRendererHold = createHold();

    hold.take();
    hold.take();
    hold.release();
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.HOLD)).toHaveLength(1);
    expect(listMockInvokeCalls(ERenderCommand.RELEASE_HOLD)).toEqual([{ hold: 4 }]);
  });

  it("releases a hold let go before the renderer answered it, once it answers", async () => {
    // A tool left at once still lets the GPU go: the hold it asked for is released as soon as it exists.
    const answer: IDeferred<number> = deferred();

    mockIpcCommands({ [ERenderCommand.HOLD]: () => answer.promise });

    const hold: NativeRendererHold = createHold();

    hold.take();
    hold.release();
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.RELEASE_HOLD)).toHaveLength(0);

    answer.resolve(9);
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.RELEASE_HOLD)).toEqual([{ hold: 9 }]);
  });

  describe("refused", () => {
    muteConsole("warn");

    it("releases nothing for a hold the renderer refused, and may hold again", async () => {
      mockIpcCommands({
        [ERenderCommand.HOLD]: () => Promise.reject(new Error("No renderer hold number is left")),
      });

      const hold: NativeRendererHold = createHold();

      hold.take();
      hold.release();
      await flush();

      expect(listMockInvokeCalls(ERenderCommand.RELEASE_HOLD)).toHaveLength(0);

      hold.take();

      expect(listMockInvokeCalls(ERenderCommand.HOLD)).toHaveLength(2);
    });
  });
});
