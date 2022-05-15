import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Amm } from "../target/types/amm";
import {
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  Account,
  Mint,
  getAccount,
  transfer,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Automated Market Maker", () => {
  // Configure the client to use the local cluster.

  const token_A_decimal = 2;
  const token_B_decimal = 3;
  const token_A_padded = Number("1".padEnd(token_A_decimal + 1, "0"));
  const token_B_padded = Number("1".padEnd(token_B_decimal + 1, "0"));
  const token_A_amount = 1000 * token_A_padded;
  const token_B_amount = 1000 * token_B_padded;
  const pool_token_A_amount = 50;
  const pool_token_B_amount = 50;
  const user_token_A_amount = 10;
  const user_token_B_amount = 10;
  const contract_owner = anchor.web3.Keypair.generate();
  const ammAccount = anchor.web3.Keypair.generate();
  const user = anchor.web3.Keypair.generate();
  let mint_A: anchor.web3.PublicKey;
  let mint_B: anchor.web3.PublicKey;
  let contract_tokenAcc_A: Account;
  let contract_tokenAcc_B: Account;
  let user_token_A_acc: Account;
  let user_token_B_acc: Account;
  let tokenAcc_A: Account;
  let tokenAcc_B: Account;
  let authority: anchor.web3.PublicKey;
  let bumpSeed: number;

  const connection = new anchor.web3.Connection("http://127.0.0.1:8899");

  const contract_wallet = new anchor.Wallet(contract_owner);

  const provider = new anchor.AnchorProvider(connection, contract_wallet, {
    preflightCommitment: "processed",
  });

  anchor.setProvider(provider);
  const program = anchor.workspace.Amm as Program<Amm>;

  it("Create token A and B", async () => {
    const airdropSig = await connection.requestAirdrop(
      contract_owner.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 10
    );

    await connection.confirmTransaction(airdropSig);

    const airdropSig1 = await connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 10
    );

    await connection.confirmTransaction(airdropSig1);
    // Start Create Mint and Contract Token Acc A and mint into it
    mint_A = await createMint(
      connection,
      contract_owner,
      contract_owner.publicKey,
      null,
      token_A_decimal
    );

    contract_tokenAcc_A = await getOrCreateAssociatedTokenAccount(
      connection,
      contract_wallet.payer,
      mint_A,
      contract_wallet.publicKey,
      true
    );
    console.log("contract_tokenAcc_A ", contract_tokenAcc_A.address.toBase58());

    await mintTo(
      connection,
      contract_wallet.payer,
      mint_A,
      contract_tokenAcc_A.address,
      contract_wallet.publicKey,
      token_A_amount
    );
    // End

    // Create token acc A and B authority
    [authority, bumpSeed] = await anchor.web3.PublicKey.findProgramAddress(
      [ammAccount.publicKey.toBuffer()],
      program.programId
    );
    console.log("Authority ", authority.toBase58());

    // Create token account A address and add authority as a owner
    tokenAcc_A = await getOrCreateAssociatedTokenAccount(
      connection,
      contract_wallet.payer,
      mint_A,
      authority,
      true
    );

    console.log("TokenAcc_a ", tokenAcc_A.address.toBase58());

    // Add some token to A token account
    await transfer(
      connection,
      contract_wallet.payer,
      contract_tokenAcc_A.address,
      tokenAcc_A.address,
      contract_owner.publicKey,
      pool_token_A_amount * token_A_padded
    );

    mint_B = await createMint(
      connection,
      contract_owner,
      contract_owner.publicKey,
      null,
      token_B_decimal
    );

    contract_tokenAcc_B = await getOrCreateAssociatedTokenAccount(
      connection,
      contract_wallet.payer,
      mint_B,
      contract_wallet.publicKey
    );
    console.log("contract_tokenAcc_B ", contract_tokenAcc_B.address.toBase58());

    await mintTo(
      connection,
      contract_wallet.payer,
      mint_B,
      contract_tokenAcc_B.address,
      contract_wallet.publicKey,
      token_B_amount
    );

    tokenAcc_B = await getOrCreateAssociatedTokenAccount(
      connection,
      contract_wallet.payer,
      mint_B,
      authority,
      true
    );

    // Add some token to B token account
    await transfer(
      connection,
      contract_wallet.payer,
      contract_tokenAcc_B.address,
      tokenAcc_B.address,
      contract_owner.publicKey,
      pool_token_B_amount * token_B_padded
    );

    console.log("TokenAcc_b ", tokenAcc_B.address.toBase58());

    // Get or create associated token account of User for mint A
    user_token_A_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mint_A,
      user.publicKey
    );
    console.log("user_token_A_acc ", user_token_A_acc.address.toBase58());

    // Add some token to User Token account for swap testing
    await transfer(
      connection,
      contract_wallet.payer,
      contract_tokenAcc_A.address,
      user_token_A_acc.address,
      contract_owner.publicKey,
      user_token_A_amount * token_A_padded
    );

    // Get or create associated token account of User for mint B
    user_token_B_acc = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      mint_B,
      user.publicKey
    );
    console.log("user_token_B_acc ", user_token_B_acc.address.toBase58());

    // await transfer(
    //   connection,
    //   contract_wallet.payer,
    //   contract_tokenAcc_B.address,
    //   user_token_B_acc.address,
    //   contract_owner.publicKey,
    //   user_token_A_amount * token_B_padded
    // );
  });

  it("Is initialized!", async () => {
    // //add you test
    try {
      const txId = await program.methods
        .initialize()
        .accounts({
          ammAccount: ammAccount.publicKey,
          signer: provider.wallet.publicKey,
          mintA: mint_A,
          mintB: mint_B,
          tokenA: tokenAcc_A.address,
          tokenB: tokenAcc_B.address,
          authority,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([ammAccount])
        .rpc();
      console.log("TxId ", txId);
    } catch (error) {
      console.log("Error ", error);
    }
  });

  it("Fetch Automated Market Maker account details", async () => {
    const state = await program.account.amm.fetch(ammAccount.publicKey);
    assert(contract_owner.publicKey.equals(state.authKey)); // contract owner is the owner of amm acount
    assert(tokenAcc_A.address.equals(state.tokenA)); // token A account address is equal to tokenA account
    assert(tokenAcc_B.address.equals(state.tokenB)); // token B account address is equal to tokenB account
    assert(mint_A.equals(state.mintA)); // check mint A address
    assert(mint_B.equals(state.mintB)); // check mint A address
    assert(token_A_decimal == state.tokenADecimal);
    assert(token_B_decimal == state.tokenBDecimal);
    assert(state.constant == pool_token_A_amount * pool_token_B_amount);
    assert(true == state.isInitialized);
  });

  it("transfer user token B to token pool B account!", async () => {
    // //add you test
    const token_amount = 10;
    try {
      const txId = await program.methods
        .swapTransferToken(new anchor.BN(token_amount))
        .accounts({
          signer: user.publicKey,
          ammAccount: ammAccount.publicKey,
          authority: authority,
          from: user_token_A_acc.address,
          tokenAccA: tokenAcc_A.address,
          tokenAccB: tokenAcc_B.address,
          to: user_token_B_acc.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      console.log("transfer user token A to token pool A account", txId);
      const state = await program.account.amm.fetch(ammAccount.publicKey);
      const token_A_acc_info = await getAccount(connection, state.tokenA);
      const t_token_a_amount = Number(token_A_acc_info.amount);
      const token_B_acc_info = await getAccount(connection, state.tokenB);
      const t_token_b_amount = Number(token_B_acc_info.amount);

      const user_token_B_acc_info = await getAccount(
        connection,
        user_token_B_acc.address
      );
      const t_user_token_B_amount = Number(user_token_B_acc_info.amount);

      let new_pool_token_a_amount = pool_token_A_amount + token_amount;
      let ratio = pool_token_A_amount * pool_token_B_amount;
      let new_pool_token_b_amount = ratio / new_pool_token_a_amount;
      let new_user_token_b_acc = pool_token_B_amount - new_pool_token_b_amount;
      // console.log(
      //   "new_pool_token_b_amount",
      //   new_pool_token_b_amount * token_B_padded
      // );
      // console.log("t_token_b_amount", t_token_b_amount);
      // console.log(
      //   "new_user_token_b_acc",
      //   new_user_token_b_acc * token_B_padded
      // );
      // console.log("t_user_token_B_amount", t_user_token_B_amount.toFixed());
      assert(
        (new_pool_token_b_amount * token_B_padded).toFixed() ==
          t_token_b_amount.toFixed()
      );
      assert(
        (new_user_token_b_acc * token_B_padded).toFixed() ==
          t_user_token_B_amount.toFixed()
      );
    } catch (error) {
      console.log("Error:transfer user token A to token pool A account", error);
    }
  });

  // it("transfer user token A to token pool A account!", async () => {
  //   // //add you test
  //   const token_amount = 4;
  //   try {
  //     const txId = await program.methods
  //       .swapTransferToken(new anchor.BN(token_amount))
  //       .accounts({
  //         signer: user.publicKey,
  //         ammAccount: ammAccount.publicKey,
  //         authority: authority,
  //         from: user_token_B_acc.address,
  //         tokenAccA: tokenAcc_A.address,
  //         tokenAccB: tokenAcc_B.address,
  //         to: user_token_A_acc.address,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .signers([user])
  //       .rpc();
  //     console.log("transfer user token B to token pool B account", txId);
  //   } catch (error) {
  //     console.log("Error:transfer user token B to token pool B account", error);
  //   }
  // });

  it("Is add token!", async () => {
    const state = await program.account.amm.fetch(ammAccount.publicKey);
    const token_A_acc_info = await getAccount(connection, state.tokenA);
    const t_token_a_amount = Number(token_A_acc_info.amount);
    const token_B_acc_info = await getAccount(connection, state.tokenB);
    const t_token_b_amount = Number(token_B_acc_info.amount);

    // add you test
    const token_amount = 10;
    try {
      const txId = await program.methods
        .addToken(new anchor.BN(token_amount))
        .accounts({
          ammAccount: ammAccount.publicKey,
          authKey: contract_owner.publicKey,
          from: contract_tokenAcc_A.address,
          to: tokenAcc_A.address,
          tokenAccA: tokenAcc_A.address,
          tokenAccB: tokenAcc_B.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();
      console.log("Add token ", txId);
      const new_state = await program.account.amm.fetch(ammAccount.publicKey);
      console.log("Constant validation ", new_state.constant);
      const normalize_token_a =
        t_token_a_amount / token_A_padded + token_amount;
      const normalize_token_b = t_token_b_amount / token_B_padded;
      console.log(
        "Constant previous state",
        normalize_token_a * normalize_token_b
      );
      assert(
        new_state.constant.toFixed() ==
          (normalize_token_a * normalize_token_b).toFixed()
      );
    } catch (error) {
      console.log("Error add token", error);
    }
  });
});
