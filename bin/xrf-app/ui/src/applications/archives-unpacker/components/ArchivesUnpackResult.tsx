import { formatBytes, formatDuration } from "@xrf/format";
import { ArchiveUnpackResult } from "@xrf/ipc/types/xrf-pack";
import { Nullable } from "@xrf/types";
import { CommandResult, ICommandResultStat } from "@xrf/ui/command-result/CommandResult";
import { CommandResultPathList } from "@xrf/ui/command-result/CommandResultPathList";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { EApplicationId } from "@/core/routing/application";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";

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
