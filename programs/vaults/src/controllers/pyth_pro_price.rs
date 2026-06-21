use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use pyth_lazer_solana_contract::{
    self,
    protocol::{
        message::SolanaMessage,
        payload::{PayloadData, PayloadPropertyValue},
    },
};

use crate::{
    constants::PRECISION,
    state::{TokenOracleConfig, TokenOracleConfigError, TokenWhitelist, VaultError},
};

#[derive(Clone, Copy, Debug)]
pub struct OraclePrice {
    pub feed_id: u32,
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

pub struct PythProPriceMap {
    channel_id: u8,
    prices: Vec<OraclePrice>,
}

pub struct PythProVerificationAccounts<'a, 'info> {
    pub payer: &'a Signer<'info>,
    pub storage: &'a Account<'info, pyth_lazer_solana_contract::Storage>,
    pub treasury: &'a AccountInfo<'info>,
    pub system_program: &'a Program<'info, System>,
    pub instructions_sysvar: &'a AccountInfo<'info>,
    pub program: &'a Program<'info, pyth_lazer_solana_contract::program::PythLazerSolanaContract>,
}

pub struct VaultTokenInfoForPythPro<'info> {
    pub token_mint: Pubkey,
    pub token_balance: u64,
    pub token_decimals: u8,
    pub oracle_config: Account<'info, TokenOracleConfig>,
}

pub fn verify_pyth_pro_message<'a, 'info>(
    accounts: PythProVerificationAccounts<'a, 'info>,
    message_data: Vec<u8>,
    ed25519_instruction_index: u16,
    signature_index: u8,
) -> Result<PythProPriceMap> {
    pyth_lazer_solana_contract::cpi::verify_message(
        CpiContext::new(
            accounts.program.to_account_info(),
            pyth_lazer_solana_contract::cpi::accounts::VerifyMessage {
                payer: accounts.payer.to_account_info(),
                storage: accounts.storage.to_account_info(),
                treasury: accounts.treasury.to_account_info(),
                system_program: accounts.system_program.to_account_info(),
                instructions_sysvar: accounts.instructions_sysvar.to_account_info(),
            },
        ),
        message_data.clone(),
        ed25519_instruction_index,
        signature_index,
    )?;

    parse_pyth_pro_price_message(&message_data)
}

pub fn parse_pyth_pro_price_message(message_data: &[u8]) -> Result<PythProPriceMap> {
    let message = SolanaMessage::deserialize_slice(message_data)
        .map_err(|_| error!(VaultError::InvalidOracleMessage))?;
    let payload = PayloadData::deserialize_slice_le(&message.payload)
        .map_err(|_| error!(VaultError::InvalidOracleMessage))?;

    let mut prices = Vec::with_capacity(payload.feeds.len());
    for feed in payload.feeds {
        let mut price = None;
        let mut conf = None;
        let mut exponent = None;
        let mut publish_time = Some(payload.timestamp_us.as_secs() as i64);

        for property in feed.properties {
            match property {
                PayloadPropertyValue::Price(value) => {
                    price = value.map(|p| p.mantissa_i64());
                }
                PayloadPropertyValue::Confidence(value) => {
                    conf = value.map(|p| p.mantissa_i64().unsigned_abs());
                }
                PayloadPropertyValue::Exponent(value) => {
                    exponent = Some(value as i32);
                }
                PayloadPropertyValue::FeedUpdateTimestamp(value) => {
                    publish_time = value.map(|t| t.as_secs() as i64);
                }
                _ => {}
            }
        }

        prices.push(OraclePrice {
            feed_id: feed.feed_id.0,
            price: price.ok_or(VaultError::OraclePriceNotFound)?,
            conf: conf.ok_or(VaultError::OraclePriceNotFound)?,
            exponent: exponent.ok_or(VaultError::InvalidOracleMessage)?,
            publish_time: publish_time.ok_or(VaultError::InvalidOracleMessage)?,
        });
    }

    Ok(PythProPriceMap {
        channel_id: payload.channel_id.0,
        prices,
    })
}

impl PythProPriceMap {
    pub fn get_checked_price(
        &self,
        oracle_config: &TokenOracleConfig,
        token_mint: &Pubkey,
    ) -> Result<OraclePrice> {
        oracle_config.verify_pyth_pro(token_mint)?;
        require!(
            self.channel_id == oracle_config.pyth_pro_channel_id,
            VaultError::InvalidOracleChannel
        );
        let price = self
            .prices
            .iter()
            .find(|price| price.feed_id == oracle_config.pyth_pro_price_feed_id)
            .copied()
            .ok_or(VaultError::OraclePriceNotFound)?;

        validate_oracle_price(
            &price,
            oracle_config.max_age_seconds,
            oracle_config.confidence_threshold_bps,
        )?;

        Ok(price)
    }
}

pub fn validate_oracle_price(
    price: &OraclePrice,
    max_age_seconds: u64,
    confidence_threshold_bps: u64,
) -> Result<()> {
    require!(price.price > 0, VaultError::InvalidOraclePrice);

    let now = Clock::get()?.unix_timestamp;
    let age = now
        .checked_sub(price.publish_time)
        .ok_or(VaultError::OraclePriceTooOld)?;
    require!(
        age >= 0 && age as u64 <= max_age_seconds,
        VaultError::OraclePriceTooOld
    );

    let conf_ratio = price
        .conf
        .checked_mul(10u64.pow(4))
        .ok_or(VaultError::NumericOverflow)?
        .checked_div(price.price.unsigned_abs())
        .ok_or(VaultError::NumericOverflow)?;
    require!(
        conf_ratio < confidence_threshold_bps,
        VaultError::PriceConfidenceTooLow
    );

    Ok(())
}

pub fn transform_oracle_price_to_aum_decimals(price: &OraclePrice) -> Result<u64> {
    let mut value = (price.price.unsigned_abs() as u128)
        .checked_mul(PRECISION as u128)
        .ok_or(VaultError::NumericOverflow)?;

    if price.exponent < 0 {
        value = value
            .checked_div(10u128.pow((-price.exponent) as u32))
            .ok_or(VaultError::NumericOverflow)?;
    } else {
        value = value
            .checked_mul(10u128.pow(price.exponent as u32))
            .ok_or(VaultError::NumericOverflow)?;
    }

    value
        .try_into()
        .map_err(|_| error!(VaultError::NumericOverflow))
}

pub fn parse_vault_balances_for_pyth_pro<'info>(
    remaining_accounts: &'info [AccountInfo<'info>],
    whitelist: &Account<'info, TokenWhitelist>,
    vault_key: &Pubkey,
) -> Result<Vec<VaultTokenInfoForPythPro<'info>>> {
    require!(
        remaining_accounts.len() % 3 == 0,
        VaultError::InvalidRemainingAccounts
    );

    let mut vault_token_infos = Vec::new();

    for chunk in remaining_accounts.chunks(3) {
        let mint = InterfaceAccount::<'info, Mint>::try_from(&chunk[0])?;
        let token_account: InterfaceAccount<'info, TokenAccount> =
            InterfaceAccount::try_from(&chunk[1])?;
        let oracle_config: Account<'_, TokenOracleConfig> = Account::try_from(&chunk[2])?;

        require!(
            mint.key() == token_account.mint,
            VaultError::MintAndTokenAccountMismatch
        );
        require!(
            *vault_key == token_account.owner,
            VaultError::VaultAndTokenAccountMismatch
        );
        require!(
            whitelist.is_whitelisted(&mint.key()),
            crate::state::TokenWhitelistError::TokenNotWhitelisted
        );

        let expected_oracle_config = Pubkey::find_program_address(
            &[TokenOracleConfig::SEED, mint.key().as_ref()],
            &crate::ID,
        )
        .0;
        require!(
            expected_oracle_config == oracle_config.key(),
            TokenOracleConfigError::InvalidOracleConfigPda
        );
        oracle_config.verify_pyth_pro(&mint.key())?;

        vault_token_infos.push(VaultTokenInfoForPythPro {
            token_mint: mint.key(),
            token_balance: token_account.amount,
            token_decimals: mint.decimals,
            oracle_config,
        });
    }

    Ok(vault_token_infos)
}
