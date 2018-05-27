pub enum Statement {
    InsertWallet,
    GetWalletID,
    DeleteWallet,
    InsertRecord,
    DeleteRecord,
    UpdateRecordValue,
    CheckRecordExists,
    UpdateTags,
    GetMetadata,
    SetMetadata,
    GetAllRecords
}

impl Statement {
    pub fn as_str(&self) -> &str {
        match self {
            &Statement::InsertWallet => "INSERT INTO wallets(name, metadata) VALUES (:name, :metadata)",
            &Statement::GetWalletID => "SELECT id FROM wallets WHERE name = :name",
            &Statement::DeleteWallet => "DELETE FROM wallets WHERE name = :name",
            &Statement::InsertRecord => "INSERT INTO items (type, name, value, tags, wallet_id) VALUE (:type, :name, :value, :tags, :wallet_id)",
            &Statement::DeleteRecord => "DELETE FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::UpdateRecordValue => "UPDATE items SET value = :value WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::CheckRecordExists => "SELECT COUNT(id) FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::UpdateTags => "UPDATE items SET tags = :tags WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::GetMetadata => "SELECT metadata FROM wallets WHERE id = :wallet_id",
            &Statement::SetMetadata => "UPDATE wallets SET metadata = :metadata WHERE id = :wallet_id",
            &Statement::GetAllRecords => "SELECT type, name, value, tags FROM items WHERE wallet_id = :wallet_id",
        }
    }
}
