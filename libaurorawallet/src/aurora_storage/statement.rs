pub enum Statement {
    InsertRecord,
    GetRecordID,
    DeleteRecord,
    UpdateRecordValue,
    CheckRecordExists,
    InsertPlaintextTag,
    InsertEncryptedTag,
    UpdatePlaintextTag,
    UpdateEncryptedTag,
    CheckPlaintextTagExists,
    CheckEncryptedTagExists,
    DeletePlaintextTag,
    DeleteEncryptedTag,
}

impl Statement {
    pub fn as_str(&self) -> &str {
        match self {
            &Statement::InsertRecord => "INSERT INTO items (type, name, value, wallet_id) VALUE (:type, :name, :value, :wallet_id)",
            &Statement::GetRecordID => "SELECT id FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::DeleteRecord => "DELETE FROM items WHERE wallet_id = :wallet_id AND type = :type AND name = :name",
            &Statement::UpdateRecordValue => "UPDATE items SET value = :value WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::CheckRecordExists => "SELECT COUNT(id) FROM items WHERE type = :type AND name = :name AND wallet_id = :wallet_id",
            &Statement::InsertPlaintextTag => "INSERT INTO tags_plaintext (name, value, item_id) VALUE (:name, :value, :item_id)",
            &Statement::InsertEncryptedTag => "INSERT INTO tags_encrypted (name, value, item_id) VALUE (:name, :value, :item_id)",
            &Statement::UpdatePlaintextTag => "UPDATE tags_plaintext set value = :value WHERE name = :name AND item_id = :item_id",
            &Statement::UpdateEncryptedTag => "UPDATE tags_encrypted set value = :value WHERE name = :name AND item_id = :item_id",
            &Statement::CheckPlaintextTagExists => "SELECT COUNT(item_id) FROM tags_plaintext WHERE name = :name AND item_id = :item_id",
            &Statement::CheckEncryptedTagExists => "SELECT COUNT(item_id) FROM tags_encrypted WHERE name = :name AND item_id = :item_id",
            &Statement::DeletePlaintextTag => "DELETE FROM tags_plaintext WHERE name = :name AND item_id = :item_id",
            &Statement::DeleteEncryptedTag => "DELETE FROM tags_encrypted WHERE name = :name AND item_id = :item_id",
        }
    }
}
