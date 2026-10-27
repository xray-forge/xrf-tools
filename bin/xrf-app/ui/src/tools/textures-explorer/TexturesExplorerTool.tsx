import { useInjection } from "@wirestate/react";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
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
  const isOpen: boolean = catalogService.isBrowsing || selectionService.selected.value !== null;

  if (!catalogService.catalog.isRestored) {
    return <ToolLoader />;
  }

  if (!isOpen) {
    return <TexturesExplorerOpenForm />;
  }

  return <TexturesExplorerWorkspace data-testid={dataTestId} />;
}
