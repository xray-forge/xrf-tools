import { Alert } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { createTexturesExplorerPanels } from "@/applications/textures-explorer/components/editor/panels/textures-panels";
import { TexturePreview } from "@/applications/textures-explorer/components/editor/preview/TexturePreview";
import { selectUnreadTexturesLtx } from "@/applications/textures-explorer/lib/texture-catalog";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureCatalog } from "@/core/bindings/types/xrf-app";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { useEditorStatus } from "@/core/shell/EditorStatusContext";
import { useEditorPanels } from "@/core/shell/panel/context";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeTexturesStatus } from "./TexturesEditor.utils";

/**
 * The open session: the texture on screen, the tree it was chosen from, and what its descriptor declares.
 */
export function TexturesEditor({
  "data-testid": dataTestId = "textures-editor",
  id,
  className,
}: BaseComponentProps = {}): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);

  const [isLtxNoticeDismissed, setLtxNoticeDismissed] = useState<boolean>(false);

  const catalog: Nullable<TextureCatalog> = texturesService.catalog.value;
  const isBrowsing: boolean = texturesService.isBrowsing;

  // Named rather than counted silently: a root set declaring one is a root set whose bumps this surface may be
  // reporting incompletely, and only the person looking at it can tell whether that matters.
  const texturesLtx: Nullable<string> = selectUnreadTexturesLtx(catalog);
  const isLtxNoticeShown: boolean = Boolean(texturesLtx) && !isLtxNoticeDismissed;

  const onClose = useCallback(() => void texturesService.close(), [texturesService]);

  useEditorPanels(() => createTexturesExplorerPanels(isBrowsing), [isBrowsing]);

  // Counted after the fold, so this says the same number the tree does: the listing holds an entry per engine
  // reference, and the halves of a declared pair are not rows a person can count.
  useEditorStatus(
    describeTexturesStatus(catalog, texturesService.nodes.length, texturesService.summaries.value?.length ?? 0)
  );

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={<EditorToolbar subtitle={texturesService.selectedReference ?? undefined} onBack={onClose} />}
      banner={
        isLtxNoticeShown ? (
          <Alert
            severity={"info"}
            closeText={"Dismiss textures.ltx notice"}
            onClose={() => setLtxNoticeDismissed(true)}
          >
            {`Bump and detail declarations are read from .thm files only. ${texturesLtx} declares them too, and is ` +
              "not read here."}
          </Alert>
        ) : null
      }
    >
      <TexturePreview />
    </EditorLayout>
  );
}
