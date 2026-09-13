import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

/**
 * Where a located asset physically sits, in one line meant to be read beside its engine path.
 *
 * @param container - Container the backend resolved the asset out of.
 * @returns The host path of a loose file, or the volume an archived entry sits in.
 */
export function describeAssetContainer(container: XrayAssetContainer): string {
  return container.kind === "directory"
    ? `${container.root}${LOGICAL_PATH_SEPARATOR}${container.relativePath}`
    : container.path;
}

/**
 * Whether a located asset is loose on disk rather than packed into a volume.
 *
 * @param container - Container the backend resolved the asset out of.
 * @returns Whether the bytes sit in a file of their own.
 */
export function isLooseContainer(container: XrayAssetContainer): boolean {
  return container.kind === "directory";
}
