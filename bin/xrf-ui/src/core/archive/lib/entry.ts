import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";

/**
 * One file the explorer browses, whichever subject it came from.
 *
 * A volume set answers with an entry of a name table; a mounted world answers with whichever copy of an engine path
 * wins. Both are a name and a size, which is everything the tree, the filter, the header and the preview gate need —
 * so both `ArchiveFileDescriptor` and `ArchiveWorldEntry` satisfy this without being converted into anything.
 */
export interface IArchiveEntry {
  /** Engine path of the file, backslash separated. */
  name: string;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** Whether the entry is a volume's directory record rather than a file with bytes. */
  isDirectory?: boolean;
  /** Where the winning copy physically sits. */
  container?: XrayAssetContainer;
}
