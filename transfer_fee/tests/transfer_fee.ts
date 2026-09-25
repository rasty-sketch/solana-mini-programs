import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TransferFee } from "../target/types/transfer_fee";

describe("transfer_fee", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.transferFee as Program<TransferFee>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
