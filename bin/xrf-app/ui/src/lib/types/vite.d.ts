import "vite/client";

declare global {
  const __REPOSITORY_URL__: string;

  /**
   * Name of the module this token appears in, substituted at build time.
   */
  const __MODULE_NAME__: string;
}

declare module "@/lib/types/vite" {}
