use solana_program::pubkey::Pubkey;

pub trait AccountStorage {
    type Account;

    fn get(&self, pubkey: &Pubkey) -> Option<Self::Account>;

    fn put(&mut self, pubkey: &Pubkey, account: Self::Account);

    fn touch(&mut self, pubkey: &Pubkey);
}
