import { ContainerDefinition } from "@/lib/container/container-definition";
import { GamedataVerifierService } from "@/tools/gamedata-verifier/services/gamedata-verifier.service";

export { GamedataVerifierTool as Component } from "./GamedataVerifierTool";

export const container: ContainerDefinition = { bindings: [GamedataVerifierService] };
