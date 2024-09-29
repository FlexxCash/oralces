use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetType {
    JupSOL,
    MSOL,
    BSOL,
    HSOL,
    JitoSOL,
    VSOL,
    SOL,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub struct AssetTypeWrapper {
    pub asset_type: AssetType,
}

impl AssetTypeWrapper {
    pub fn new(asset_type: AssetType) -> Self {
        Self { asset_type }
    }
}

impl Default for AssetTypeWrapper {
    fn default() -> Self {
        Self { asset_type: AssetType::SOL }
    }
}

impl From<AssetType> for AssetTypeWrapper {
    fn from(asset_type: AssetType) -> Self {
        Self::new(asset_type)
    }
}

impl AssetTypeWrapper {
    pub fn get(&self) -> AssetType {
        self.asset_type
    }
}