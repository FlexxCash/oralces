import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";

interface PriceOracleHeader {
  lastGlobalUpdate: anchor.BN;
  emergencyStop: boolean;
  authority: anchor.web3.PublicKey;
  switchboardProgramId: anchor.web3.PublicKey;
  bump: number;
}

interface PriceData {
  price: number;
  lastPrice: number;
  lastUpdateTime: anchor.BN;
  apy: number;
}

interface PriceOracleData {
  priceData: PriceData[];
  bump: number;
}

describe("price_oracle", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programId = new anchor.web3.PublicKey("GqYaWFTAy3dTNZ8zRb9EyWLqTQ4gRHUUwCCuD5GmRihY");
  const program = anchor.workspace.Oracles as Program<any>;

  let priceOracleHeaderPda: anchor.web3.PublicKey;
  let priceOracleDataPda: anchor.web3.PublicKey;
  let oracleFeed: anchor.web3.PublicKey;
  let solOracleFeed: anchor.web3.PublicKey;
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

      oracleFeed = new anchor.web3.PublicKey("4NiWaTuje7SVe9DN1vfnX7m1qBC7DnUxwRxbdgEDUGX1");
      solOracleFeed = new anchor.web3.PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");
      switchboardProgram = new anchor.web3.PublicKey("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
    } catch (error) {
      console.error("Error in before hook:", error);
      throw error;
    }
  });

  it("Initializes the price oracle", async () => {
    try {
      await program.methods.initialize(switchboardProgram)
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          authority: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda) as PriceOracleHeader;
      const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda) as PriceOracleData;
      assert.isNotNull(headerAccount, "Header account should not be null");
      assert.isNotNull(dataAccount, "Data account should not be null");
      assert.equal(headerAccount.switchboardProgramId.toBase58(), switchboardProgram.toBase58(), "Switchboard program ID should match");
    } catch (error) {
      console.error("Error initializing price oracle:", error);
      throw error;
    }
  });

  it("Updates prices and APYs for all assets", async () => {
    try {
      await program.methods.updatePricesAndApys()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: oracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const dataAccount = await program.account.priceOracleData.fetch(priceOracleDataPda) as PriceOracleData;
      assert.equal(dataAccount.priceData.length, 7, "Should have updated 7 assets (6 + SOL)");
      dataAccount.priceData.forEach((data, index) => {
        if (index < 6) {
          assert.isTrue(data.price > 0, `Asset ${index} price should be greater than 0`);
          assert.isTrue(data.apy > 0, `Asset ${index} APY should be greater than 0`);
        }
      });
    } catch (error) {
      console.error("Error updating prices and APYs for all assets:", error);
      throw error;
    }
  });

  it("Updates SOL price", async () => {
    try {
      const beforeUpdate = await program.account.priceOracleData.fetch(priceOracleDataPda) as PriceOracleData;
      const beforeSolPrice = beforeUpdate.priceData[6].price;

      await program.methods.updateSolPrice()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: solOracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const afterUpdate = await program.account.priceOracleData.fetch(priceOracleDataPda) as PriceOracleData;
      const afterSolPrice = afterUpdate.priceData[6].price;

      assert.isNotNull(afterSolPrice, "SOL price should not be null");
      assert.isTrue(afterSolPrice > 0, "SOL price should be greater than 0");
      assert.notEqual(beforeSolPrice, afterSolPrice, "SOL price should have changed");
      
      console.log(`SOL price updated from ${beforeSolPrice} to ${afterSolPrice}`);
    } catch (error) {
      console.error("Error updating SOL price:", error);
      throw error;
    }
  });

  it("Gets current price for JupSOL", async () => {
    try {
      const tx = await program.methods.getCurrentPrice({ jupSol: {} })
        .accounts({
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

  it("Gets current price for SOL", async () => {
    try {
      const tx = await program.methods.getCurrentPrice({ sol: {} })
        .accounts({
          data: priceOracleDataPda,
        })
        .rpc();

      const txLogs = await provider.connection.getTransaction(tx, { commitment: 'confirmed' });
      assert.isTrue(txLogs.meta.logMessages.some(log => log.includes("Current price for SOL:")), "Transaction logs should include current price for SOL");
    } catch (error) {
      console.error("Error getting current price for SOL:", error);
      throw error;
    }
  });

  it("Gets current APY for JupSOL", async () => {
    try {
      const tx = await program.methods.getCurrentApy({ jupSol: {} })
        .accounts({
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

  it("Sets and checks emergency stop", async () => {
    try {
      await program.methods.setEmergencyStop(true)
        .accounts({
          header: priceOracleHeaderPda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda) as PriceOracleHeader;
      assert.isTrue(headerAccount.emergencyStop, "Emergency stop should be set to true");
    } catch (error) {
      console.error("Error setting or checking emergency stop:", error);
      throw error;
    }
  });

  it("Fails to update prices and APYs when emergency stop is active", async () => {
    try {
      await program.methods.updatePricesAndApys()
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

  it("Fails to update SOL price when emergency stop is active", async () => {
    try {
      await program.methods.updateSolPrice()
        .accounts({
          header: priceOracleHeaderPda,
          data: priceOracleDataPda,
          oracleFeed: solOracleFeed,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      assert.fail("Should have thrown an error");
    } catch (error) {
      assert.include(error.toString(), "Emergency stop activated");
    }
  });

  it("Fails to update prices and APYs with unauthorized user", async () => {
    const unauthorizedUser = anchor.web3.Keypair.generate();
    try {
      await program.methods.updatePricesAndApys()
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

  it("Updates Switchboard program ID", async () => {
    const newSwitchboardProgramId = anchor.web3.Keypair.generate().publicKey;
    try {
      await program.methods.updateSwitchboardProgramId(newSwitchboardProgramId)
        .accounts({
          header: priceOracleHeaderPda,
          authority: provider.wallet.publicKey,
        })
        .rpc();

      const headerAccount = await program.account.priceOracleHeader.fetch(priceOracleHeaderPda) as PriceOracleHeader;
      assert.equal(headerAccount.switchboardProgramId.toBase58(), newSwitchboardProgramId.toBase58(), "Switchboard program ID should be updated");
    } catch (error) {
      console.error("Error updating Switchboard program ID:", error);
      throw error;
    }
  });
});