import { Alert } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useState } from "react";

import { TextureCatalog } from "@/core/bindings/types/xrf-app";
import { IEditorPanel } from "@/core/shell/panel/context";
import { TexturePreviewLayout } from "@/core/textures/components/workspace/TexturePreviewLayout";
import { selectUnreadTexturesLtx } from "@/core/textures/lib/texture-catalog";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { createTexturesExplorerPanels } from "./panels/textures-explorer-panels";
import { describeTexturesStatus } from "./TexturesExplorerWorkspace.utils";

/**
 * The browsing session: the tree, the texture chosen from it, and what the root set as a whole came to.
 */
export function TexturesExplorerWorkspace({
  "data-testid": dataTestId = "textures-explorer-workspace",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);

  const [isLtxNoticeDismissed, setLtxNoticeDismissed] = useState<boolean>(false);

  const catalog: Nullable<TextureCatalog> = catalogService.catalog.value;
  const isBrowsing: boolean = catalogService.isBrowsing;

  // Named rather than counted silently: a root set declaring one is a root set whose bumps this surface may be
  // reporting incompletely, and only the person looking at it can tell whether that matters.
  const texturesLtx: Nullable<string> = selectUnreadTexturesLtx(catalog);
  const isLtxNoticeShown: boolean = Boolean(texturesLtx) && !isLtxNoticeDismissed;

  const panels: Array<IEditorPanel> = useMemo(() => createTexturesExplorerPanels(isBrowsing), [isBrowsing]);
  const onBack = useCallback(() => void catalogService.close(), [catalogService]);

  return (
    <TexturePreviewLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      panels={panels}
      // Counted after the fold, so this says the same number the tree does: the listing holds an entry per engine
      // reference, and the halves of a declared pair are not rows a person can count.
      status={describeTexturesStatus(catalog, catalogService.nodes.length, catalogService.summaries.value?.length ?? 0)}
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
      onBack={onBack}
    />
  );
}
