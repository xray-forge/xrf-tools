/**
 * Checks whether this webview runs on macOS.
 *
 * `navigator.platform` is deprecated but still the only synchronous answer; `userAgentData` is asynchronous and
 * absent outside Chromium. The user agent is the fallback, which is also what carries the answer under a test runner.
 *
 * @returns Whether the host is macOS.
 */
export function isApplePlatform(): boolean {
  return typeof navigator !== "undefined" && /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);
}
