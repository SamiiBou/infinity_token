#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::clock::Epoch;

    // Fonction utilitaire pour créer un AccountInfo pour les tests
    fn create_account_info<'a>(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo::new(
            key,
            is_signer,
            is_writable,
            lamports,
            data,
            owner,
            false,
            Epoch::default(),
        )
    }

    #[test]
    fn test_initialize_mint() {
        let program_id = Pubkey::new_unique();
        let mint_key = Pubkey::new_unique();
        let mut mint_account = vec![0; Mint::LEN];
        let mut mint_lamports = 0;
        let mint_authority_key = Pubkey::new_unique();
        let freeze_authority_key = Pubkey::new_unique();
        let mut rent_sysvar = vec![0; Rent::default().try_to_vec().unwrap().len()];

        let accounts = vec![
            create_account_info(&mint_key, false, true, &mut mint_lamports, &mut mint_account, &program_id),
            create_account_info(&mint_authority_key, true, false, &mut 0, &mut [], &program_id),
            create_account_info(&freeze_authority_key, false, false, &mut 0, &mut [], &program_id),
            create_account_info(&solana_program::sysvar::rent::id(), false, false, &mut 0, &mut rent_sysvar, &solana_program::system_program::id()),
        ];

        let instruction_data = [0, 9]; // InitializeMint with 9 decimals

        let result = process_instruction(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());
    }

}
