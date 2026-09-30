import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { ArchiveUnpackResult } from "@/core/ipc/types/xrf-pack";
import { EApplicationId } from "@/core/routing/application";
import { CommandResult, ICommandResultStat } from "@/core/ui/command-result/CommandResult";
import { CommandResultPathList } from "@/core/ui/command-result/CommandResultPathList";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { formatBytes } from "@/lib/memory/format";

interface IArchivesUnpackResultProps extends BaseComponentProps {
  result: ArchiveUnpackResult;
  /** Where the run was told to write. The result's rendered `destination` is display text, not an address. */
  outputPath: Nullable<string>;
}

export function ArchivesUnpackResult({
  "data-testid": dataTestId = "archives-unpack-result",
  id,
  className,
  result,
  outputPath,
}: IArchivesUnpackResultProps): ReactElement {
  const stats: Array<ICommandResultStat> = useMemo(
    () => [
      { label: "archives", value: result.archives.length },
      { label: "unpacked", value: formatBytes(result.unpackedSize) },
      { label: "prepare", value: formatDuration(result.prepareDuration) },
      { label: "unpack", value: formatDuration(result.unpackDuration) },
      { label: "elapsed", value: formatDuration(result.duration) },
    ],
    [result]
  );

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={`Unpacked ${result.archives.length} archive(s) to ${result.destination}`}
      tone={"success"}
      stats={stats}
      actions={
        <RevealPathButton application={EApplicationId.ARCHIVES_UNPACKER} path={outputPath} label={"Show output"} />
      }
    >
      <CommandResultPathList
        paths={result.archives}
        column={"Archive"}
        emptyLabel={"No archives were unpacked."}
        searchPlaceholder={"Filter by archive"}
      />
    </CommandResult>
  );
}
