import { ReactElement, useMemo } from "react";

import { listPayloadSharersOf } from "@/core/archive/files";
import { ArchiveFileDescriptor, ArchiveSharedPayload } from "@/core/bindings/types/xrf-archive";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveSharedPayloadDetailProps extends BaseComponentProps {
  descriptor: ArchiveFileDescriptor;
  sharedPayloads: AsyncState<Array<ArchiveSharedPayload>>;
}

/**
 * Which other entries read the selected file's bytes.
 */
export function ArchiveSharedPayloadDetail({
  "data-testid": dataTestId,
  id,
  className,
  descriptor,
  sharedPayloads,
}: IArchiveSharedPayloadDetailProps): ReactElement {
  const others: Array<string> = listPayloadSharersOf(sharedPayloads.value ?? [], descriptor);

  const description: string = useMemo(() => {
    if (sharedPayloads.isLoading) {
      return "Deriving from the name table...";
    }

    if (sharedPayloads.error) {
      return "Could not derive which entries read the same bytes.";
    }

    if (!others.length) {
      return "No other entry reads these bytes";
    }

    return `${
      others.length
    } other entr${others.length === 1 ? "y reads" : "ies read"} these bytes, derived from equal descriptors`;
  }, [others.length, sharedPayloads.error, sharedPayloads.isLoading]);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <EditorPanelProperty label={"Shared payload"} value={description} />

      {others.map((name: string) => (
        <EditorPanelProperty key={name} label={"Entry"} value={name} isMonospace />
      ))}
    </div>
  );
}
