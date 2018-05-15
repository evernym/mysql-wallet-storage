pub enum Statement {
    InsertRecord,
    GetRecordID,
    DeleteRecord,
    UpdateRecordValue,
    CheckRecordExists,
    UpsertPlaintextTag,
    UpsertEncryptedTag,
    CheckPlaintextTagExists,
    CheckEncryptedTagExists,
    DeletePlaintextTag,
    DeleteEncryptedTag,
    DeleteAllPlaintextTags,
    DeleteAllEncryptedTags,
    GetMetadata,
    SetMetadata,
}

impl Statement {
    pub fn as_str(&self) -> &str {
        match self {
            &Statement::InsertRecord => "INSERT INTO items (type, name, value, wallet_id) VALUE (:type, :name, :value, :wallet_id)",
            &Statement::GetRecordID => "SELECT id FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::DeleteRecord => "DELETE FROM items WHERE wallet_id = :wallet_id AND type = :type AND name = :name",
            &Statement::UpdateRecordValue => "UPDATE items SET value = :value WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::CheckRecordExists => "SELECT COUNT(id) FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::UpsertPlaintextTag => "INSERT INTO tags_plaintext (name, value, item_id) VALUE (:name, :value, :item_id) ON DUPLICATE KEY UPDATE value = :value",
            &Statement::UpsertEncryptedTag => "INSERT INTO tags_encrypted (name, value, item_id) VALUE (:name, :value, :item_id) ON DUPLICATE KEY UPDATE value = :value",
            &Statement::CheckPlaintextTagExists => "SELECT COUNT(item_id) FROM tags_plaintext WHERE name = :name AND item_id = :item_id",
            &Statement::CheckEncryptedTagExists => "SELECT COUNT(item_id) FROM tags_encrypted WHERE name = :name AND item_id = :item_id",
            &Statement::DeletePlaintextTag => "DELETE FROM tags_plaintext WHERE name = :name AND item_id = :item_id",
            &Statement::DeleteEncryptedTag => "DELETE FROM tags_encrypted WHERE name = :name AND item_id = :item_id",
            &Statement::DeleteAllPlaintextTags => "DELETE FROM tags_plaintext WHERE item_id = :item_id",
            &Statement::DeleteAllEncryptedTags => "DELETE FROM tags_encrypted WHERE item_id = :item_id",
            &Statement::GetMetadata => "SELECT metadata FROM wallets WHERE id = :wallet_id",
            &Statement::SetMetadata => "UPDATE wallets SET metadata = :metadata WHERE id = :wallet_id"
        }
    }
}
