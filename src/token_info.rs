use solana_program::pubkey::Pubkey;
use solana_program::program_pack::{Pack, Sealed};
use solana_program::program_error::ProgramError;

pub struct TokenInfo {
    pub total_supply: u64,
    pub team_allocation: u64,
    pub investors_allocation: u64,
    pub liquidity_reserve: u64,
    pub development_reserve: u64,
    pub community_rewards: u64,
    pub strategic_reserve: u64,
    pub mint_authority: Pubkey,
    pub mint: Pubkey,
}

impl Sealed for TokenInfo {}

impl Pack for TokenInfo {
    const LEN: usize = 8 * 7 + 32 * 2;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut offset = 0;
        dst[offset..offset+8].copy_from_slice(&self.total_supply.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.team_allocation.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.investors_allocation.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.liquidity_reserve.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.development_reserve.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.community_rewards.to_le_bytes());
        offset += 8;
        dst[offset..offset+8].copy_from_slice(&self.strategic_reserve.to_le_bytes());
        offset += 8;
        dst[offset..offset+32].copy_from_slice(self.mint_authority.as_ref());
        offset += 32;
        dst[offset..offset+32].copy_from_slice(self.mint.as_ref());
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut offset = 0;
        let total_supply = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let team_allocation = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let investors_allocation = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let liquidity_reserve = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let development_reserve = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let community_rewards = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let strategic_reserve = u64::from_le_bytes(src[offset..offset+8].try_into().unwrap());
        offset += 8;
        let mint_authority = Pubkey::from_bytes(&src[offset..offset+32]);
        offset += 32;
        let mint = Pubkey::from_bytes(&src[offset..offset+32]);

        Ok(TokenInfo {
            total_supply,
            team_allocation,
            investors_allocation,
            liquidity_reserve,
            development_reserve,
            community_rewards,
            strategic_reserve,
            mint_authority,
            mint,
        })
    }
}