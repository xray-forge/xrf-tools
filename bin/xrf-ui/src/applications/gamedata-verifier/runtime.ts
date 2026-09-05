import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { GamedataVerifierApplication as Component } from "./GamedataVerifierApplication";

export const container: ContainerDefinition = { bindings: [GamedataVerifierService] };
