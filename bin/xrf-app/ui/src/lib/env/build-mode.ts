/**
 * Checks whether this bundle was built for development.
 *
 * @returns Whether the bundle uses the development environment.
 */
export function isDevelopmentBuild(): boolean {
  return process.env.NODE_ENV !== "production";
}
