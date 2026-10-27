import { formatBytesPair, formatCount, formatDuration } from "@xrf/format";
import { ArchivePackResult } from "@xrf/ipc/types/xrf-pack";
import { CommandResult, CommandResultPathList, ICommandResultStat } from "@xrf/ui/command-result";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useMemo } from "react";

import { EApplicationId } from "@/core/routing/application";
import { RevealPathButton } from "@/core/ui/reveal/RevealPathButton";

interface IArchivesPackResultProps extends BaseComponentProps {
  result: ArchivePackResult;
}

export function ArchivesPackResult({
  "data-testid": dataTestId = "archives-pack-result",
  id,
  className,
  result,
}: IArchivesPackResultProps): ReactElement {
  const stats: Array<ICommandResultStat> = useMemo(() => {
    // One unit for both sizes, so the compression ratio stays readable at a glance.
    const [sizeSource, sizeWritten] = formatBytesPair(result.sizeSource, result.sizeWritten);

    return [
      { label: "volumes", value: result.volumes.length },
      { label: "packed", value: result.filesTotal },
      { label: "compressed", value: result.filesCompressed },
      { label: "stored", value: result.filesStored },
      // Aliased and skipped are the two counts that explain a surprising size or a missing file.
      { label: "aliased", value: result.filesAliased },
      { label: "skipped", value: result.filesSkipped },
      { label: "source", value: sizeSource },
      { label: "written", value: sizeWritten },
      { label: "elapsed", value: formatDuration(result.duration) },
    ];
  }, [result]);

  return (
    <CommandResult
      data-testid={dataTestId}
      id={id}
      className={className}
      headline={`Packed ${formatCount(result.filesTotal)} file(s) into ${result.volumes.length} volume(s)`}
      tone={"success"}
      stats={stats}
      // The first volume rather than the output directory, so the file manager opens with something
      // this run produced selected rather than with whatever else lives there.
      actions={
        <RevealPathButton
          application={EApplicationId.ARCHIVES_PACKER}
          path={result.volumes[0] ?? null}
          label={"Show volumes"}
        />
      }
    >
      <CommandResultPathList
        paths={result.volumes}
        column={"Volume"}
        emptyLabel={"No volumes were written."}
        searchPlaceholder={"Filter by volume"}
      />
    </CommandResult>
  );
}
