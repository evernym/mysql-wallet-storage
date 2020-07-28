import asyncio
import os

from indy import wallet
from .mysql_lib_loader import load_mysql_storage
import yaml
import argparse
import json
import tempfile
from mysql import connector


async def export_from_sqlite(config) -> str:
    print("Exporting old wallet...")
    wallet_cfg = {
        "id": config["wallet"]["name"]
    }
    wallet_creds = {
        "key": str(config["wallet"]["key"]),
        "key_derivation_method": config["wallet"]["key_derivation_method"]
    }
    name = os.path.join(tempfile.gettempdir(), next(tempfile._get_candidate_names()))
    export_cfg = {
        "path": name,
        "key": str(config["wallet"]["export_key"])
    }
    handle = await wallet.open_wallet(json.dumps(wallet_cfg), json.dumps(wallet_creds))
    await wallet.export_wallet(handle, json.dumps(export_cfg))
    await wallet.close_wallet(handle)
    return name


async def import_to_mysql(config, name):
    print("Importing wallet...")
    load_mysql_storage()
    wallet_cfg = {
        "id": config["wallet"]["name"],
        "storage_type": "mysql",
        "storage_config": {
            "db_name": config["mysql"]["db_name"],
            "port": config["mysql"]["port"],
            "write_host": config["mysql"]["host"],
            "read_host": config["mysql"]["host"],
        }
    }
    wallet_creds = {
        "key": str(config["wallet"]["key"]),
        "key_derivation_method": config["wallet"]["key_derivation_method"],
        "storage_credentials": {
            "user": config["mysql"]["user"],
            "pass": config["mysql"]["password"],
        }
    }
    import_cfg = {
        "path": name,
        "key": str(config["wallet"]["export_key"])
    }
    await wallet.import_wallet(json.dumps(wallet_cfg), json.dumps(wallet_creds), json.dumps(import_cfg))


def create_database(config):
    print("Checking database for required tables...")
    cnx = connector.connect(host=config["mysql"]["host"],
                            database=config["mysql"]["db_name"],
                            user=config["mysql"]["user"],
                            password=config["mysql"]["password"])
    cursor = cnx.cursor()
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS `wallets` (
        `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
        `name` VARCHAR(1024) NOT NULL,
        `metadata` VARCHAR(10240) NOT NULL,
        PRIMARY KEY (`id`),
        UNIQUE KEY `wallet_name` (`name`)
    ) ENGINE=InnoDB DEFAULT CHARSET=ascii;
    ''')
    cursor.execute('''
    CREATE TABLE IF NOT EXISTS `items` (
        `id` BIGINT(20) NOT NULL AUTO_INCREMENT,
        `wallet_id` BIGINT(20) NOT NULL,
        `type` VARCHAR(128) NOT NULL,
        `name` VARCHAR(1024) NOT NULL,
        `value` LONGBLOB NOT NULL,
        `tags` JSON NOT NULL,
        PRIMARY KEY (`id`),
        UNIQUE KEY `ux_items_wallet_id_type_name` (`wallet_id`, `type`, `name`),
        CONSTRAINT `fk_items_wallet_id` FOREIGN KEY (`wallet_id`)
            REFERENCES `wallets` (`id`)
            ON DELETE CASCADE
            ON UPDATE CASCADE
    ) ENGINE=InnoDB DEFAULT CHARSET=ascii
    ''')



def parse_config(config_path):
    with open(config_path) as f:
        data = yaml.load(f, Loader=yaml.FullLoader)
    return data


def report_config_changes(config):
    print("Done! Now you need to update your VCX config with these values:")
    print("\"wallet_type\": \"mysql\",")
    cfg = json.dumps({
        "db_name": config["mysql"]["db_name"],
        "port": config["mysql"]["port"],
        "write_host": config["mysql"]["host"],
        "read_host": config["mysql"]["host"],
    })
    print("\"storage_config\": {},".format(json.dumps(cfg)))
    creds = json.dumps({
        "user": config["mysql"]["user"],
        "pass": config["mysql"]["password"],
    })
    print("\"storage_credentials\": {}".format(json.dumps(creds)))


def migrate():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", help="path to config with wallet and database configuration", default="config.yml")
    args = parser.parse_args()
    loop = asyncio.get_event_loop()

    config = parse_config(args.config)
    name = loop.run_until_complete(export_from_sqlite(config))
    create_database(config)
    loop.run_until_complete(import_to_mysql(config, name))
    report_config_changes(config)


if __name__ == "__main__":
    migrate()
