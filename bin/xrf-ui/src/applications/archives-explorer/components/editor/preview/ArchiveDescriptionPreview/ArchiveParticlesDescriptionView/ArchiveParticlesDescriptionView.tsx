import { Typography } from "@mui/material";
import { ReactElement, useMemo, useState } from "react";

import {
  ArchiveDescribeScope,
  ArchiveParticlesDescription,
  ArchiveParticlesEffect,
  ArchiveParticlesGroup,
} from "@/core/ipc/types/xrf-app";
import { EditorFilterInput } from "@/core/shell/editor/EditorFilterInput";
import { EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ArchiveDescriptionLayout } from "../ArchiveDescriptionLayout";
import { filterByName } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionRow } from "../ArchiveDescriptionRow";
import { ArchiveParticlesEffectRow } from "./ArchiveParticlesEffectRow";
import { ArchiveParticlesGroupRow } from "./ArchiveParticlesGroupRow";

interface IArchiveParticlesDescriptionViewProps extends BaseComponentProps {
  description: ArchiveParticlesDescription;
  scope: ArchiveDescribeScope;
}

/**
 * The particle library: the emitters it defines and the sequences that play them.
 */
export function ArchiveParticlesDescriptionView({
  "data-testid": dataTestId = "archive-particles-description-view",
  id,
  className,
  description,
  scope,
}: IArchiveParticlesDescriptionViewProps): ReactElement {
  const [filter, setFilter] = useState<string>("");

  const { library, effects, groups } = description;

  const matchedEffects: Array<ArchiveParticlesEffect> = useMemo(
    () => filterByName(effects, filter, (effect: ArchiveParticlesEffect) => effect.name),
    [effects, filter]
  );

  const matchedGroups: Array<ArchiveParticlesGroup> = useMemo(
    () => filterByName(groups, filter, (group: ArchiveParticlesGroup) => group.name),
    [filter, groups]
  );

  const isFiltered: boolean = Boolean(filter.trim());

  return (
    <ArchiveDescriptionLayout data-testid={dataTestId} id={id} className={className}>
      <EditorPanelSection title={"Library"} isFirst>
        <ArchiveDescriptionRow
          label={"Effects"}
          value={`${library.effects}`}
          caption={`Built from ${library.actions} actions between them`}
        />

        <ArchiveDescriptionRow
          label={"Groups"}
          value={`${library.groups}`}
          caption={
            library.undefinedEffects
              ? `Naming ${library.undefinedEffects} ${library.undefinedEffects === 1 ? "effect" : "effects"} this library does not define`
              : "Every effect they name is defined here"
          }
        />

        <ArchiveDescriptionRow
          label={"Textures"}
          value={`${library.textures}`}
          caption={
            library.absentTextures ? `${library.absentTextures} not held by what is open` : "All held by what is open"
          }
        />

        <ArchiveDescriptionRow label={"Version"} value={`${library.version}`} />
      </EditorPanelSection>

      <div className={"px-4 pt-2"}>
        <EditorFilterInput
          query={filter}
          placeholder={"Filter effects and groups"}
          ariaLabel={"Filter effects and groups"}
          onQueryChange={setFilter}
        />
      </div>

      <EditorPanelSection
        title={isFiltered ? `Effects (${matchedEffects.length} of ${effects.length})` : `Effects (${effects.length})`}
        caption={"In the order the library declares them"}
      >
        {matchedEffects.length ? (
          matchedEffects.map((effect: ArchiveParticlesEffect, index: number) => (
            <ArchiveParticlesEffectRow key={`${index}-${effect.name}`} effect={effect} scope={scope} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No effect of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>

      <EditorPanelSection
        title={isFiltered ? `Groups (${matchedGroups.length} of ${groups.length})` : `Groups (${groups.length})`}
        caption={"Each slot names an effect, and the effects it starts alongside it"}
      >
        {matchedGroups.length ? (
          matchedGroups.map((group: ArchiveParticlesGroup, index: number) => (
            <ArchiveParticlesGroupRow key={`${index}-${group.name}`} group={group} />
          ))
        ) : (
          <Typography className={"text-text-disabled"} variant={"body2"}>
            No group of this library is named that.
          </Typography>
        )}
      </EditorPanelSection>
    </ArchiveDescriptionLayout>
  );
}
