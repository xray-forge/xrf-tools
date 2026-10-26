import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";
import { default as GroupsIcon } from "@mui/icons-material/Groups";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as RouteIcon } from "@mui/icons-material/Route";
import { default as ScienceIcon } from "@mui/icons-material/Science";
import { ReactElement, ReactNode, useMemo } from "react";
import { NavigateFunction, useLocation, useNavigate } from "react-router-dom";

import { EditorSideMenu, IEditorSideMenuItem } from "@/core/shell/editor/EditorSideMenu";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ISpawnChunkSection {
  label: string;
  icon: ReactNode;
  path: string;
}

const SECTIONS: Array<ISpawnChunkSection> = [
  { label: "Header", icon: <InfoIcon />, path: "header" },
  { label: "Alife", icon: <GroupsIcon />, path: "alife" },
  { label: "Artefacts", icon: <ScienceIcon />, path: "artefacts" },
  { label: "Patrols", icon: <RouteIcon />, path: "patrols" },
  { label: "Graph", icon: <AccountTreeIcon />, path: "graph" },
];

const EDITOR_PATH: string = "/spawn-editor";

export function SpawnEditorMenu({
  "data-testid": dataTestId = "spawn-editor-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const navigate: NavigateFunction = useNavigate();
  const { pathname } = useLocation();

  const sections: Array<IEditorSideMenuItem> = useMemo(
    () =>
      SECTIONS.map((it: ISpawnChunkSection) => ({
        icon: it.icon,
        label: it.label,
        isSelected: pathname.startsWith(`${EDITOR_PATH}/${it.path}`),
        onClick: () => navigate(`${EDITOR_PATH}/${it.path}`, { replace: true }),
      })),
    [navigate, pathname]
  );

  return <EditorSideMenu data-testid={dataTestId} id={id} className={className} sections={sections} />;
}
