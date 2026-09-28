export {};

declare global {
  interface Window {
    /**
     * Kind of build the binary recorded, set by its initialization script before any page script runs.
     */
    readonly __XRF_BUILD_KIND__?: string;
  }
}
