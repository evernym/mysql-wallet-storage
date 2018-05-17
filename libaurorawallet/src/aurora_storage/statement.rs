pub enum Statement {
    InsertWallet,
    GetWalletID,
    DeleteWallet,
    InsertRecord,
    GetRecordID,
    DeleteRecord,
    UpdateRecordValue,
    CheckRecordExists,
    UpsertPlaintextTag,
    UpsertEncryptedTag,
    DeletePlaintextTag,
    DeleteEncryptedTag,
    DeleteAllPlaintextTags,
    DeleteAllEncryptedTags,
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
            &Statement::InsertRecord => "INSERT INTO items (type, name, value, wallet_id) VALUE (:type, :name, :value, :wallet_id)",
            &Statement::GetRecordID => "SELECT id FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::DeleteRecord => "DELETE FROM items WHERE wallet_id = :wallet_id AND type = :type AND name = :name",
            &Statement::UpdateRecordValue => "UPDATE items SET value = :value WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::CheckRecordExists => "SELECT COUNT(id) FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::UpsertPlaintextTag => "INSERT INTO tags_plaintext (name, value, item_id) VALUE (:name, :value, :item_id) ON DUPLICATE KEY UPDATE value = :value",
            &Statement::UpsertEncryptedTag => "INSERT INTO tags_encrypted (name, value, item_id) VALUE (:name, :value, :item_id) ON DUPLICATE KEY UPDATE value = :value",
            &Statement::DeletePlaintextTag => "DELETE FROM tags_plaintext WHERE name = :name AND item_id = :item_id",
            &Statement::DeleteEncryptedTag => "DELETE FROM tags_encrypted WHERE name = :name AND item_id = :item_id",
            &Statement::DeleteAllPlaintextTags => "DELETE FROM tags_plaintext WHERE item_id = :item_id",
            &Statement::DeleteAllEncryptedTags => "DELETE FROM tags_encrypted WHERE item_id = :item_id",
            &Statement::GetMetadata => "SELECT metadata FROM wallets WHERE id = :wallet_id",
            &Statement::SetMetadata => "UPDATE wallets SET metadata = :metadata WHERE id = :wallet_id",
            &Statement::GetAllRecords => "SELECT type, name, value, \
                                          CONCAT( \
                                            '{', \
                                                CONCAT_WS( \
                                                    ',', \
                                                    (select group_concat(concat(json_quote(name), ':', json_quote(value))) from tags_encrypted WHERE item_id = i.id), \
                                                    (select group_concat(concat(json_quote(concat('~', name)), ':', json_quote(value))) from tags_plaintext WHERE item_id = i.id) \
                                                ), \
                                            '}') tags \
                                          FROM items i \
                                          WHERE wallet_id = :wallet_id"
        }
    }
}

