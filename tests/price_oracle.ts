import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Idl } from "@coral-xyz/anchor";
import { assert } from "chai";

// 定義自定義接口
interface PriceOracleProgram extends Program<Idl> {
  account: {
    priceOracleHeader: any;
    priceOracleData: any;
  };
}

describe("price_oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 手動初始化程序
  const programId = new anchor.web3.PublicKey("AtguUUsGDXry7onmb7QqDK4DLwquRkQPsXX1CJTjZsUy");
  const idl = require("../target/idl/oracles.json") as Idl;
  const program = new anchor.Program(idl, programId, provider) as PriceOracleProgram;

  let priceOracleHeaderPda: anchor.web3.PublicKey;
  let priceOracleDataPda: anchor.web3.PublicKey;
  let oracleFeed: anchor.web3.PublicKey;

  before(async () => {
    const [headerPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("price_oracle_header")],
      program.programId
    );
    priceOracleHeaderPda = headerPda;

    const [dataPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("price_oracle_data")],
      program.programId
    );
    priceOracleDataPda = dataPda;

    // 使用 JupSOL 的 feed address 作為示例
    oracleFeed = new anchor.web3.PublicKey("3zkXukqF4CBSUAq55uAx1CnGrzDKk3cVAesJ4WLpSzgA");
  });

  it("Initializes the price oracle", async () => {
    await program.methods.initialize()
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda);
    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    assert.isNotNull(headerAccount);
    assert.isNotNull(dataAccount);
    assert.isEmpty(dataAccount.priceData);
    assert.isEmpty(dataAccount.assetTypes);
  });

  it("Updates price for JupSOL", async () => {
    await program.methods.updatePrice({ jupSol: {} })
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        oracleFeed: oracleFeed,
      })
      .rpc();

    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    const jupSolPrice = dataAccount.priceData.find((_, index) => 
      'jupSol' in (dataAccount.assetTypes[index] as { [key: string]: any })
    );
    assert.isNotNull(jupSolPrice);
    assert.isTrue(jupSolPrice.price > 0);
  });

  it("Updates APY for JupSOL", async () => {
    await program.methods.updateApy({ jupSol: {} })
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        oracleFeed: oracleFeed,
      })
      .rpc();

    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    const jupSolApy = dataAccount.priceData.find((_, index) => 
      'jupSol' in (dataAccount.assetTypes[index] as { [key: string]: any })
    );
    assert.isNotNull(jupSolApy);
    assert.isTrue(jupSolApy.apy > 0);
  });

  it("Gets current price for JupSOL", async () => {
    const tx = await program.methods.getCurrentPrice({ jupSol: {} })
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
      })
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current price for JupSOL:")));
  });

  it("Gets current APY for JupSOL", async () => {
    const tx = await program.methods.getCurrentApy({ jupSol: {} })
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
      })
      .rpc();

    const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
    assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current APY for JupSOL:")));
  });

  it("Fails to update price too frequently", async () => {
    try {
      await program.methods.updatePrice({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
        })
        .rpc();
      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include((error as Error).message, "Price update is too frequent");
    }
  });

  it("Gets SOL price", async () => {
    const tx = await program.methods.getSolPrice()
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
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