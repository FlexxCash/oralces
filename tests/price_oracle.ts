import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

describe("price_oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // 更新為 lib.rs 中聲明的程序 ID
  const programId = new anchor.web3.PublicKey("GxkpGSztczkz7hNPUcN8XbZjnyMYqW8YMmTqtKVA579e");
  const program = anchor.workspace.Oracles as Program<any>;

  let priceOracleHeaderPda: anchor.web3.PublicKey;
  let priceOracleDataPda: anchor.web3.PublicKey;
  let oracleFeed: anchor.web3.PublicKey;
  let switchboardProgram: anchor.web3.PublicKey;

  before(async () => {
    try {
      const [headerPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("price_oracle_header")],
        programId
      );
      priceOracleHeaderPda = headerPda;

      const [dataPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("price_oracle_data")],
        programId
      );
      priceOracleDataPda = dataPda;

      oracleFeed = new anchor.web3.PublicKey("3zkXukqF4CBSUAq55uAx1CnGrzDKk3cVAesJ4WLpSzgA");
      switchboardProgram = new anchor.web3.PublicKey("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
    } catch (error) {
      console.error("Error in before hook:", error);
      throw error;
    }
  });

  it("Initializes the price oracle", async () => {
    try {
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
      assert.isNotNull(headerAccount, "Header account should not be null");
      assert.isNotNull(dataAccount, "Data account should not be null");
      assert.isEmpty((dataAccount as any).priceData, "Price data should be empty");
      assert.isEmpty((dataAccount as any).assetTypes, "Asset types should be empty");
    } catch (error) {
      console.error("Error initializing price oracle:", error);
      throw error;
    }
  });

  it("Updates price for JupSOL", async () => {
    try {
      await program.methods.updatePrice({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
      const jupSolPrice = (dataAccount as any).priceData.find((_, index: number) => 
        (dataAccount as any).assetTypes[index].jupSol !== undefined
      );
      assert.isNotNull(jupSolPrice, "JupSOL price should not be null");
      assert.isTrue(jupSolPrice.price > 0, "JupSOL price should be greater than 0");
    } catch (error) {
      console.error("Error updating price for JupSOL:", error);
      throw error;
    }
  });

  it("Updates APY for JupSOL", async () => {
    try {
      await program.methods.updateApy({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          switchboardProgram: switchboardProgram,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
      const jupSolApy = (dataAccount as any).priceData.find((_, index: number) => 
        (dataAccount as any).assetTypes[index].jupSol !== undefined
      );
      assert.isNotNull(jupSolApy, "JupSOL APY should not be null");
      assert.isTrue(jupSolApy.apy > 0, "JupSOL APY should be greater than 0");
    } catch (error) {
      console.error("Error updating APY for JupSOL:", error);
      throw error;
    }
  });

  it("Gets current price for JupSOL", async () => {
    try {
      const tx = await program.methods.getCurrentPrice({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current price for JupSOL:")), "Transaction logs should include current price for JupSOL");
    } catch (error) {
      console.error("Error getting current price for JupSOL:", error);
      throw error;
    }
  });

  it("Gets current APY for JupSOL", async () => {
    try {
      const tx = await program.methods.getCurrentApy({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current APY for JupSOL:")), "Transaction logs should include current APY for JupSOL");
    } catch (error) {
      console.error("Error getting current APY for JupSOL:", error);
      throw error;
    }
  });

  it("Gets SOL price", async () => {
    try {
      const tx = await program.methods.getSolPrice()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current SOL price:")), "Transaction logs should include current SOL price");
    } catch (error) {
      console.error("Error getting SOL price:", error);
      throw error;
    }
  });

  it("Gets USDC price", async () => {
    try {
      const tx = await program.methods.getUsdcPrice()
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current USDC price: $1.00")), "Transaction logs should include current USDC price");
    } catch (error) {
      console.error("Error getting USDC price:", error);
      throw error;
    }
  });

  it("Sets and checks emergency stop", async () => {
    try {
      await program.methods.setEmergencyStop(true)
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const tx = await program.methods.checkEmergencyStop()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Emergency stop status: true")), "Emergency stop should be set to true");
    } catch (error) {
      console.error("Error setting or checking emergency stop:", error);
      throw error;
    }
  });

  it("Fails to update price when emergency stop is active", async () => {
    try {
      await program.methods.updatePrice({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Emergency stop activated");
    }
  });

  it("Fails to update price with unauthorized user", async () => {
    const unauthorizedUser = anchor.web3.Keypair.generate();
    try {
      await program.methods.updatePrice({ jupSol: {} })
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: unauthorizedUser.publicKey,
        })
        .signers([unauthorizedUser])
        .rpc();

      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Unauthorized access");
    }
  });

  it("Updates price for multiple assets", async () => {
    // 首先關閉緊急停止
    await program.methods.setEmergencyStop(false)
      .accounts({
        header: priceOracleHeaderPda,
        data: priceOracleDataPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const assets = [{ mSol: {} }, { bSol: {} }, { hSol: {} }];
    for (const asset of assets) {
      await program.methods.updatePrice(asset)
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    }

    const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda);
    assert.isAtLeast((dataAccount as any).assetTypes.length, assets.length, "Should have updated all new assets");
  });
});