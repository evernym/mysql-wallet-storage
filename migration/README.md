## Migration Tool Guide

### How to Use
To use the script:
* Setup a MySQL server with an empty database.
* Install the libmysqlstorage package.
* Setup a Python 3 environment.
* Download the migration script from this repository.
* From the directory of the migration script, install dependencies with `pip install -r            requirements.txt`
* Setup the migration configuration file `config.yml`
* Run the migration script with `python3 -m migration`


### How to Configure

There is a config file named `config.yml`. It has some fields that need to be configured:
```
wallet:
  name: w1 // wallet name
  key: 1 // wallet key
  export_key: 23 // export key, any value you want
  key_derivation_method: ARGON2I_MOD // derivation method you have set
mysql:
  host: 0.0.0.0 // mysql host
  port: 3306 // mysql port
  user: root // username of the mysql user that can create tables
  password: root // password for this user
  db_name: wallet // db name that will be used for wallets
```


### Requirements

* Python 3
* MySQL -- a server should be already running
* libindy
* libmysqlstorage
