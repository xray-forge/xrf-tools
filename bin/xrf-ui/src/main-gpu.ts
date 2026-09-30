/**
 * Asks for a WebGPU adapter as the document starts and lets it go, inlined into `index.html` before the first paint. The
 * GPU process sets WebGPU up once a run, about half a second on its main thread, which also draws the window: asked
 * here, it does so while the window is still hidden and the bundle boots, not as the first viewport opens.
 */
((): void => {
  const gpu = (navigator as Navigator & { gpu?: { requestAdapter(options?: object): Promise<unknown> } }).gpu;

  // The renderer's own options, so what the GPU process sets up is what the renderer then asks for.
  gpu?.requestAdapter({ featureLevel: "compatibility" }).catch(() => {});
})();
