import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act, render } from "@testing-library/react";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { mockInvoke, mockIsTauri } from "@xrf/ipc/testing";
import { ReactElement, StrictMode } from "react";

import { useWindowReveal } from "./use-window-reveal";

function Window(): ReactElement {
  useWindowReveal();

  return <div>window</div>;
}

function runFrame(frames: Array<FrameRequestCallback>): void {
  act(() => {
    for (const frame of frames.splice(0)) {
      frame(0);
    }
  });
}

describe("useWindowReveal", () => {
  const frames: Array<FrameRequestCallback> = [];

  beforeEach(() => {
    mockInvoke.mockClear();
    frames.length = 0;

    jest.spyOn(window, "requestAnimationFrame").mockImplementation((frame: FrameRequestCallback) => frames.push(frame));
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("reveals the window once, two frames after its first screen", () => {
    // Strict mode runs the effect twice, as a development page does.
    render(
      <StrictMode>
        <Window />
      </StrictMode>
    );

    runFrame(frames);
    expect(mockInvoke).not.toHaveBeenCalled();

    runFrame(frames);
    runFrame(frames);
    expect(mockInvoke.mock.calls).toEqual([[ESystemCommand.REVEAL_WINDOW]]);
  });

  it("does nothing outside the desktop runtime", () => {
    mockIsTauri.mockReturnValueOnce(false);

    render(<Window />);

    expect(frames).toHaveLength(0);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
