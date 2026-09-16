import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

/**
 * One file the explorer browses, whichever subject it came from.
 *
 * A volume set answers with an entry of a name table; a mounted world answers with whichever copy of an engine path
 * wins. Both are a name and a size, which is everything the tree, the filter, the header and the preview gate need —
 * so both `ArchiveFileDescriptor` and `ArchiveWorldEntry` satisfy this without being converted into anything.
 */
export interface IArchiveEntry {
  /** Name of the file, backslash separated, as its own subject spells it. */
  name: string;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** Whether the entry is a volume's directory record rather than a file with bytes. */
  isDirectory?: boolean;
  /** Where the winning copy physically sits. */
  container?: XrayAssetContainer;
}

/**
 * The engine identity an entry answers for, whichever subject named it.
 *
 * @param entry - Entry to identify.
 * @returns Its engine identity: lower case, backslash separated.
 */
export function getEntryEngineIdentity(entry: IArchiveEntry): string {
  return entry.name.split("/").join(LOGICAL_PATH_SEPARATOR).toLowerCase();
}
