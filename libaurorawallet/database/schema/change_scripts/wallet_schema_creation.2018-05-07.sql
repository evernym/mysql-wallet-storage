
/*** Wallet Schema Creation Script ***/


CREATE DATABASE IF NOT EXISTS `wallet` DEFAULT CHARACTER SET latin1;

USE `wallet`;

/*** Wallet Table Structure - Mapping of wallet names to Integer IDs. ***/

CREATE TABLE IF NOT EXISTS `wallets` (
    `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(1024) NOT NULL,
    `metadata` VARCHAR(10240) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `wallet_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

/*** Item Table Structure - Holding Item Data. ***/

CREATE TABLE IF NOT EXISTS `items` (
    `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
    `wallet_id` BIGINT(20) NOT NULL,
    `type` VARCHAR(64) NOT NULL,
    `name` VARCHAR(1024) NOT NULL,
    `value` LONGBLOB NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_items_wallet_id_type_name` (`wallet_id`, `type`, `name`),
    CONSTRAINT `fk_items_wallet_id` FOREIGN KEY (`wallet_id`)
        REFERENCES `wallets` (`id`)
        ON DELETE CASCADE
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

/*** Item Table Structure - Holding Item Data. ***/

CREATE TABLE IF NOT EXISTS `items_1` (
    `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
    `wallet_id` BIGINT(20) NOT NULL,
    `type` VARCHAR(64) NOT NULL,
    `name` VARCHAR(1024) NOT NULL,
    `value` LONGBLOB NOT NULL,
    `tags` JSON,
    PRIMARY KEY (`id`),
    UNIQUE KEY `ux_items_wallet_id_type_name_!` (`wallet_id`, `type`, `name`),
    CONSTRAINT `fk_items_wallet_id_1` FOREIGN KEY (`wallet_id`)
        REFERENCES `wallets` (`id`)
        ON DELETE CASCADE
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

/*** Encrypted Tags Table - Holding Items Encrypted Tags. ***/

CREATE TABLE IF NOT EXISTS `tags_encrypted` (
    `name` VARCHAR(256) NOT NULL,
    `value` VARCHAR(2816) NOT NULL,
    `item_id` BIGINT NOT NULL,
    PRIMARY KEY (`name`, `item_id`),
    KEY `ix_tags_encrypted_name_value` (`name`, `value`),
    CONSTRAINT `fk_tags_encrypted_item_id` FOREIGN KEY (`item_id`)
        REFERENCES `items` (`id`)
        ON DELETE CASCADE
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

/*** PlainText Tags Table - Holding Items Unencrypted String Tags. ***/

CREATE TABLE IF NOT EXISTS `tags_plaintext` (
    `name` VARCHAR(256) NOT NULL,
    `value` VARCHAR(2816) NOT NULL,
    `item_id` BIGINT NOT NULL,
    PRIMARY KEY (`name`, `item_id`),
    KEY `ix_tags_plaintext_name_value` (`name`, `value`),
    CONSTRAINT `fk_tags_plaintext_item_id` FOREIGN KEY (`item_id`)
        REFERENCES `items` (`id`)
        ON DELETE CASCADE
        ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=latin1;

