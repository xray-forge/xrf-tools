import { EditorSideMenu, IEditorSideMenuItem } from "@xrf/ui/editor/side-menu";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { GroupsIcon } from "@xrf/ui/icons/groups";
import { InfoIcon } from "@xrf/ui/icons/info";
import { RouteIcon } from "@xrf/ui/icons/route";
import { ScienceIcon } from "@xrf/ui/icons/science";
import { ReactElement, ReactNode, useMemo } from "react";
import { NavigateFunction, useLocation, useNavigate } from "react-router-dom";

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
