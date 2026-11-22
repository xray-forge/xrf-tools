import { getPathSeparator } from "@xrf/path";

/** The detail object tool's part file, which the build writes `build.details` from. */
const DETAILS_PART_FILE_NAME: string = "detail_object.part";

/**
 * The part file a scene holds its details in, in the directory of parts named after its `.level`. The build writes
 * `build.details` only from it, and the compile always runs after the build, so a scene without it has no details to
 * light.
 *
 * @param scenePath - The scene's `.level`.
 * @returns The detail part's path.
 */
export function getSceneDetailsPart(scenePath: string): string {
  return [scenePath.replace(/\.level$/i, ""), DETAILS_PART_FILE_NAME].join(getPathSeparator(scenePath));
}
