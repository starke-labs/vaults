use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use transfer_hook::{
    cpi::{
        accounts::{CloseExtraAccountMetasAccounts, InitializeExtraAccountMetasAccounts},
        close_extra_account_metas_accounts, initialize_extra_account_metas,
    },
    program::TransferHook,
};

pub fn initialize_vtoken_config<'info>(
    vtoken_is_transferrable: bool,
    manager: &Signer<'info>,
    mint: &InterfaceAccount<'info, Mint>,
    extra_account_metas: &AccountInfo<'info>,
    vtoken_config: &AccountInfo<'info>,
    transfer_hook_program: &Program<'info, TransferHook>,
    system_program: &Program<'info, System>,
) -> Result<()> {
    // TODO: Can implement From trait for `InitializeExtraAccountMetasAccounts` struct
    let initialize_extra_account_metas_accounts = InitializeExtraAccountMetasAccounts {
        manager: manager.to_account_info(),
        mint: mint.to_account_info(),
        extra_account_metas: extra_account_metas.to_account_info(),
        vtoken_config: vtoken_config.to_account_info(),
        system_program: system_program.to_account_info(),
    };
    initialize_extra_account_metas(
        CpiContext::new(
            transfer_hook_program.to_account_info(),
            initialize_extra_account_metas_accounts,
        ),
        vtoken_is_transferrable,
    )?;

    Ok(())
}

pub fn close_vtoken_config<'info>(
    manager: &Signer<'info>,
    vtoken_config: &AccountInfo<'info>,
    extra_account_metas: &AccountInfo<'info>,
    transfer_hook_program: &Program<'info, TransferHook>,
    mint: &InterfaceAccount<'info, Mint>,
) -> Result<()> {
    let close_extra_account_metas_accs = CloseExtraAccountMetasAccounts {
        manager: manager.to_account_info(),
        vtoken_config: vtoken_config.to_account_info(),
        extra_account_metas: extra_account_metas.to_account_info(),
        mint: mint.to_account_info(),
    };
    close_extra_account_metas_accounts(CpiContext::new(
        transfer_hook_program.to_account_info(),
        close_extra_account_metas_accs,
    ))?;

    Ok(())
}
