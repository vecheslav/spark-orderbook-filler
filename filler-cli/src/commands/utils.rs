use clap::ValueEnum;
use fuels::{
    crypto::SecretKey,
    prelude::{ContractId, Provider, WalletUnlocked},
};
use std::str::FromStr;

pub(crate) async fn setup(rpc: &str) -> anyhow::Result<(WalletUnlocked, Vec<WalletUnlocked>)> {
    let provider = Provider::connect(rpc).await?;
    let mnemonic = std::env::var("WALLET_MNEMONIC")?;
    let wallet = WalletUnlocked::new_from_mnemonic_phrase(&mnemonic, Some(provider.clone()))?;

    let traders = (0..5)
        .map(|i| {
            let secret_key = SecretKey::new_from_mnemonic_phrase_with_path(
                &mnemonic,
                &format!("m/44'/60'/0'/{}", i),
            )
            .unwrap();
            WalletUnlocked::new_from_private_key(secret_key, Some(provider.clone()))
        })
        .collect::<Vec<_>>();

    Ok((wallet, traders))
}

pub(crate) fn validate_contract_id(contract_id: &str) -> anyhow::Result<ContractId> {
    if contract_id.len() as u64 != 66 {
        anyhow::bail!("Invalid contract id length");
    }

    Ok(ContractId::from_str(contract_id).expect("Invalid contract id"))
}

#[derive(Clone, ValueEnum)]
pub(crate) enum IdentityType {
    /// Externally Owned Account
    Address,
    /// Contract
    Contract,
}
