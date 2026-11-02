/**
 * Records a build kind the way the binary's initialization script does, or removes it with `undefined`.
 *
 * @param kind - Value the binary would have written.
 */
export function setMockBuildKind(kind: string | undefined): void {
  if (kind === undefined) {
    resetMockBuildKind();
  } else {
    Object.defineProperty(window, "__XRF_BUILD_KIND__", { configurable: true, value: kind });
  }
}

/**
 * Leaves the document as no binary hosts it.
 */
export function resetMockBuildKind(): void {
  Reflect.deleteProperty(window, "__XRF_BUILD_KIND__");
}
