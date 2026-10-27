import { formatCount } from "@xrf/format";
import {
  ArchiveGameMtlDescription,
  ArchiveGameMtlMaterial,
  ArchiveGameMtlProperty,
} from "@xrf/ipc/types/xrf-app-archives";
import { filterByName } from "@xrf/search";
import { EditorFilterInput } from "@xrf/ui/editor/filter-input";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useMemo, useState } from "react";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveGameMtlMaterialRow } from "./ArchiveGameMtlMaterialRow";

interface IArchiveGameMtlViewProps extends BaseComponentProps {
  description: ArchiveGameMtlDescription;
}

/**
 * The game material library: what every surface is made of, and what happens where two of them meet.
 */
export function ArchiveGameMtlView({
  "data-testid": dataTestId = "archive-game-mtl-view",
  id,
  className,
  description,
}: IArchiveGameMtlViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { materials, properties } = description;

  const matched: Array<ArchiveGameMtlMaterial> = useMemo(
    () => filterByName(materials, filter, (material: ArchiveGameMtlMaterial) => material.name),
    [materials, filter]
  );

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Pairings"} isFirst>
        <ArchiveDescriptionRow
          label={"Pairs"}
          value={formatCount(description.pairs)}
          caption={
            `${formatCount(description.inheritingPairs)} name a parent pairing the editor fills their unset ` +
            "properties from"
          }
        />

        {properties.map((property: ArchiveGameMtlProperty) => (
          <ArchiveDescriptionRow
            key={property.name}
            label={property.name.charAt(0).toUpperCase() + property.name.slice(1)}
            value={formatCount(property.pairs)}
          />
        ))}

        <ArchiveDescriptionRow label={"Version"} value={`${description.version}`} />
      </EditorPanelSection>

      <EditorPanelSection
        title={
          filter.trim() ? `Materials (${matched.length} of ${materials.length})` : `Materials (${materials.length})`
        }
        caption={"A collision face stores one of these by number, which is what a level carries"}
      >
        <EditorFilterInput
          className={"mb-2"}
          ariaLabel={"Filter materials"}
          query={filter}
          placeholder={"Filter materials"}
          onQueryChange={setFilter}
        />

        {matched.length ? (
          matched.map((material: ArchiveGameMtlMaterial) => (
            <ArchiveGameMtlMaterialRow key={material.id} material={material} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No material of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
