import { jest } from "@jest/globals";

/**
 * Stubs the renderer's thread for a test of whatever starts a renderer, which has to import it afterwards: the worker
 * entry reads `import.meta.url`, which the test transform cannot, and jsdom has no offscreen canvas.
 *
 * @param createWorker - Hands each renderer started its worker, usually a stub's from `@xrf/renderer/fixtures`.
 */
export function mockRendererThread(createWorker: () => Worker): void {
  jest.doMock("@xrf/renderer/worker", () => ({ createRendererWorker: createWorker }));

  // The client only hands the result to the worker.
  HTMLCanvasElement.prototype.transferControlToOffscreen = function (): OffscreenCanvas {
    return {} as OffscreenCanvas;
  };
}
