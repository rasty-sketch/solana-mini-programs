// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Pda } from "../target/types/pda";
// import { PublicKey } from "@solana/web3.js";
// import { assert } from "chai";


// describe(


//   "pda",

//   () => {
//     const provider = anchor.AnchorProvider.env();
//     anchor.setProvider(provider)

//     const program = anchor.workspace.pda as Program<Pda>;
//     const wallet = provider.wallet;
//     console.log(` The program id here is : ${program.programId}`);
//     const [messagePda, messageBump] = PublicKey.findProgramAddressSync(
//       [Buffer.from("message"), wallet.publicKey.toBuffer()],
//       program.programId
//     );
//     it("Create Message Account", async () => {
//       try {
//         const message = "hello, world"
//         const transactionSignature = await program.methods
//           .create(message)
//           .accountsPartial({
//             user: wallet.publicKey,
//             messageAccount: messagePda,
//           })
//           .rpc({commitment:"confirmed",skipPreflight:true})

//         const messageAccount = await program.account.messageAccount.fetch(
//           messagePda,
//           "confirmed"
//         )
//         console.error("running test...")
//       }
//       catch (err) {
//         console.error("TEST FAILED:", err);
//         throw err;
//       }
//     });

//     it("Update Message Account", async () => { });

//     it("Delete Message Account", async () => { });
//   }
// );
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Pda } from "../target/types/pda";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("pda", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.pda as Program<Pda>;
  const wallet = provider.wallet;
  console.log(`The program id here is : ${program.programId}`);

  const [messagePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("message"), wallet.publicKey.toBuffer()],
    program.programId
  );

  it("Create Message Account", async () => {
    try {
      const message = "hello, world";

      await program.methods
        .create(message)
        .accountsPartial({
          user: wallet.publicKey,
          messageAccount: messagePda,
        })
        .rpc({ commitment: "confirmed", skipPreflight: true });

      const messageAccount = await program.account.messageAccount.fetch(
        messagePda,
        "confirmed"
      );

      assert.equal(messageAccount.message, message);
      assert.equal(messageAccount.user.toBase58(), wallet.publicKey.toBase58());
      console.log("Message account created with:", messageAccount.message);
    } catch (err) {
      console.error("TEST FAILED:", err);
      throw err;
    }
  });

  it("Update Message Account", async () => {
    const newMessage = "updated message!";

    await program.methods
      .update(newMessage)
      .accountsPartial({
        user: wallet.publicKey,
        messageAccount: messagePda,
      })
      .rpc({ commitment: "confirmed", skipPreflight: true });

    const messageAccount = await program.account.messageAccount.fetch(
      messagePda,
      "confirmed"
    );

    assert.equal(messageAccount.message, newMessage);
    console.log("Message updated to:", messageAccount.message);
  });

  it("Delete Message Account", async () => {
    await program.methods
      .delete()
      .accountsPartial({
        user: wallet.publicKey,
        messageAccount: messagePda,
      })
      .rpc({ commitment: "confirmed", skipPreflight: true });

    // Verify the account no longer exists
    const accountInfo = await provider.connection.getAccountInfo(messagePda);
    assert.isNull(accountInfo, "Message account should be closed");
    console.log("Message account successfully deleted");
  });
});
