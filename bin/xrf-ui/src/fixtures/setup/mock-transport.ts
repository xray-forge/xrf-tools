import { mockFetch } from "@/fixtures/mocks/bulk.mocks";

/**
 * Stands the mocked loopback transport in for the page's `fetch`, which is the only thing that reaches it.
 */
export function mockTransport(): void {
  Object.defineProperty(globalThis, "fetch", { configurable: true, value: mockFetch, writable: true });
}
