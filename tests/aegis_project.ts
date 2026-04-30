import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { AegisProject } from "../target/types/aegis_project";

describe("aegis_project", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.aegisProject as Program<AegisProject>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
