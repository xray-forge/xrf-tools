import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Binding } from "@wirestate/core";
import { makeAutoObservable } from "@wirestate/mobx";

import { ERenderViewportEvent, RenderViewportEvent } from "@/core/ipc/types/xrf-renderer";
import { BIND_POSE, IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import {
  getMockChannels,
  MockChannel,
  mockInvoke,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { VisualPreviewViewport } from "./VisualPreviewViewport";

let bindings: Array<Binding>;

function sent(command: string): Array<Record<string, unknown>> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === `plugin:render|${command}`)
    .map(([, args]) => args as Record<string, unknown>);
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

beforeEach(() => {
  resetMockInvoke();
  resetMockChannels();
  setMockInvokeResponses({ ["plugin:render|attach_viewport"]: 1 });

  const source: IVisualRenderSource = makeAutoObservable<IVisualRenderSource>(
    { hasBump: false, model: mockVisualModelViews(), pose: BIND_POSE, sessionId: "open" },
    {},
    { deep: false }
  );

  bindings = [
    VisualLoadService,
    VisualViewService,
    VisualRenderService,
    { factory: () => source, token: VISUAL_RENDER_SOURCE },
  ];
});

describe("VisualPreviewViewport", () => {
  it("attaches one native viewport, shows the open model in it, and lets it go when unmounted", async () => {
    const { unmount } = renderWithProviders(<VisualPreviewViewport />, { bindings });

    await act(flush);

    expect(sent("attach_viewport")).toHaveLength(1);
    expect(sent("show_model").map(({ sessionId }) => sessionId)).toEqual(["open"]);

    unmount();
    await flush();

    expect(sent("detach_viewport")).toHaveLength(1);
  });

  it("covers the viewport and says why when the renderer fails", async () => {
    const view: RenderResult = renderWithProviders(<VisualPreviewViewport />, { bindings });

    await act(flush);
    act(() =>
      (getMockChannels()[0] as MockChannel<RenderViewportEvent>).onmessage({
        kind: ERenderViewportEvent.FAILURE,
        message: "No GPU adapter",
      })
    );

    expect(view.getByTestId("render-failure-cover")).toHaveTextContent("No GPU adapter");
    expect(view.queryByTestId("render-frame-readout")).not.toBeInTheDocument();

    view.unmount();
  });
});
