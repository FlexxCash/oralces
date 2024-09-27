import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MySolanaProgram } from "../target/types/flexxcash_oracle";
import { PublicKey } from '@solana/web3.js';
import { assert } from "chai";

describe("price_oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MySolanaProgram as Program<MySolanaProgram>;

  let priceOraclePda: PublicKey;
  let oracleFeed: PublicKey;

  before(async () => {
    const [pda] = await PublicKey.findProgramAddress(
      [Buffer.from("price_oracle")],
      program.programId
    );
    priceOraclePda = pda;

    // 使用 JupSOL 的 feed address 作為示例
    oracleFeed = new PublicKey("3zkXukqF4CBSUAq55uAx1CnGrzDKk3cVAesJ4WLpSzgA");
  });

  it("Initializes the price oracle", async () => {
    await program.methods.initialize()
      .accounts({
        priceOracle: priceOraclePda,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.priceOracle.fetch(priceOraclePda);
    assert.isNotNull(account);
    assert.isEmpty(account.prices);
    assert.isEmpty(account.apys);
  });

  it("Updates price for JupSOL", async () => {
    await program.methods.updatePrice({ jupSol: {} })
      .accounts({
        priceOracle: priceOraclePda,
        oracleFeed: oracleFeed,
      })
      .rpc();

    const account = await program.account.priceOracle.fetch(priceOraclePda);
    const jupSolPrice = account.prices.find(([assetType]) => 'jupSol' in assetType);
    assert.isNotNull(jupSolPrice);
    assert.isTrue(jupSolPrice[1].price > 0);
  });

  it("Updates APY for JupSOL", async () => {
    await program.methods.updateApy({ jupSol: {} })
      .accounts({
        priceOracle: priceOraclePda,
        oracleFeed: oracleFeed,
      })
      .rpc();

    const account = await program.account.priceOracle.fetch(priceOraclePda);
    const jupSolApy = account.apys.find(([assetType]) => 'jupSol' in assetType);
    assert.isNotNull(jupSolApy);
    assert.isTrue(jupSolApy[1].apy > 0);
  });

  it("Gets current price for JupSOL", async () => {
    const tx = await program.methods.getCurrentPrice({ jupSol: {} })
      .accounts({
        priceOracle: priceOraclePda,
      })
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current price for JupSOL:")));
  });

  it("Gets current APY for JupSOL", async () => {
    const tx = await program.methods.getCurrentApy({ jupSol: {} })
      .accounts({
        priceOracle: priceOraclePda,
      })
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current APY for JupSOL:")));
  });

  it("Fails to update price too frequently", async () => {
    try {
      await program.methods.updatePrice({ jupSol: {} })
        .accounts({
          priceOracle: priceOraclePda,
          oracleFeed: oracleFeed,
        })
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.message, "Price update is too frequent");
    }
  });

  it("Gets SOL price", async () => {
    const tx = await program.methods.getSolPrice()
      .accounts({
        priceOracle: priceOraclePda,
      })
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current SOL price:")));
  });

  it("Gets USDC price", async () => {
    const tx = await program.methods.getUsdcPrice()
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current USDC price: $1.00")));
  });
});