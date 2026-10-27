import { ImageIcon } from "@xrf/ui/icons/image";
import { InfoIcon } from "@xrf/ui/icons/info";
import { LayersIcon } from "@xrf/ui/icons/layers";
import { TuneIcon } from "@xrf/ui/icons/tune";

import { TexturesMenu } from "@/applications/textures-explorer/components/tree/TexturesMenu";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { TextureChannelsPanel } from "@/core/textures/components/panels/TextureChannelsPanel";
import { TextureFilesPanel } from "@/core/textures/components/panels/TextureFilesPanel";
import { TextureMaterialPanel } from "@/core/textures/components/panels/TextureMaterialPanel";

/**
 * The panels the explorer offers, browsing on the left and inspection on the right.
 *
 * @param isBrowsing - Whether a root set is open.
 * @returns The panels to register.
 */
export function createTexturesExplorerPanels(isBrowsing: boolean): Array<IEditorPanel> {
  return [
    ...(isBrowsing
      ? [
          {
            icon: <ImageIcon />,
            id: "textures",
            isOpenByDefault: true,
            label: "Textures",
            render: () => <TexturesMenu />,
            side: "left" as const,
          },
        ]
      : []),
    {
      icon: <InfoIcon />,
      id: "material",
      isOpenByDefault: true,
      label: "Material",
      render: () => <TextureMaterialPanel />,
    },
    {
      icon: <LayersIcon />,
      id: "files",
      isOpenByDefault: false,
      label: "Files",
      render: () => <TextureFilesPanel />,
    },
    {
      icon: <TuneIcon />,
      id: "channels",
      isOpenByDefault: false,
      label: "Channels",
      render: () => <TextureChannelsPanel />,
    },
  ];
}
