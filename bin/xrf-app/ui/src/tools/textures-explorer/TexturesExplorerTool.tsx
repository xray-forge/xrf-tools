import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { SessionTool } from "@/core/session/components/SessionTool";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { TextureCatalogService } from "@/tools/textures-explorer/services/texture-catalog.service";

import { TexturesExplorerOpenForm } from "./components/TexturesExplorerOpenForm";
import { TexturesExplorerWorkspace } from "./components/TexturesExplorerWorkspace";

/**
 * Browse a root set of textures, or inspect one texture on its own.
 */
export function TexturesExplorerTool({
  "data-testid": dataTestId = "textures-explorer-tool",
}: BaseComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  return (
    <SessionTool
      root={catalogService.catalog}
      isOpen={catalogService.isBrowsing || selectionService.selected.value !== null}
      openForm={<TexturesExplorerOpenForm />}
      workspace={<TexturesExplorerWorkspace data-testid={dataTestId} />}
    />
  );
}
