import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as TuneIcon } from "@mui/icons-material/Tune";

import { IEditorPanel } from "@/core/shell/panel/context";
import { TextureChannelsPanel } from "@/core/textures/components/panels/TextureChannelsPanel";
import { TextureFilesPanel } from "@/core/textures/components/panels/TextureFilesPanel";
import { TextureMaterialPanel } from "@/core/textures/components/panels/TextureMaterialPanel";
import { TexturesMenu } from "@/core/textures/components/tree/TexturesMenu";

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
