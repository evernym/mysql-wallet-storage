# Indy SDK MySQL Wallet Storage Plug-In

## About

By default, the Indy SDK stores wallet data in an embedded SQLite database. It also provides an API for plugging in various types of wallet storage backends.

This project provides an Indy wallet storage plugin that allows the use of a MySQL compatible database. 

## Supported platforms

<table>
  <tbody>
    <tr>
      <th rowspan="2">Operating System</th>
      <th rowspan="2">Version</th>
      <th  colspan="2"> Architecture </th>
    </tr>
    <tr>
      <th> i386 </th>
      <th> amd64 </th>
    <tr>
      <td >Ubuntu</td>
      <td> 18.04 LTS (Bionic Beaver) </td>
      <td> &#10006; </td>
      <td> &#10004; </td>
    </tr>
  </tbody>
</table>

It has been used with:
* MySQL 5.7, MySQL 8.0
* MariaDB
* AWS RDS
* Azure Database for MySQL


## How to Use

1. Install the Debian package,
2. Setup an empty database with credentials that can be used by the application,
3. Use the database tools to setup the schema using [this SQL script](./db_scripts/schema/change_scripts/wallet_schema_creation.2018-05-07.sql).
4. Then follow the instructions for the method you use to interact with the wallet.

### LibVCX

If you are using LibVCX you need to set these config values in `vcx_init`. You can then access the wallet as normal.

```
"wallet_type": "mysql"
"storage_config": "{
    db_name: "<database name>",
    port: "<mysql db port>",
    write_host: "<mysql db write hostname>",
    read_host: "<mysql db read hostname>", // in most usecases this is the same host
}"
"storage_credentials": "{
    user: "<db username>",
    pass: "<db user password>"
}"
```

### LibIndy

To use liblndy directly, you need to call the `mysql_storage_init` function from your application code to register the plugin to libindy.

Then, when you call libindy wallet functions such as `create_wallet` or `open_wallet`, you pass in the wallet configuration and database credentials as parameters:

```
config: {
    storage_type: "mysql",
    storage_config: {
        db_name: "<database name>",
        port: "<mysql db port>",
        write_host: "<mysql db write hostname>",
        read_host: "<mysql db read hostname>", // usually the same as the write_host
    }
}
credentials: {
    storage_credentials: {
        user: "<db username>",
        pass: "<db user password>"
    }
}
```

### Migrating from SQLite to MySQL

There is a migration script available for moving from the SQLite wallet storage to MySQL. To use the script:
* Setup a MySQL server with an empty database.
* Install the libmysqlstorage package.
* Setup the configuration file `config.yml`
* Setup a Python 3 environment.


## How to Build

1. Install dependencies
 - Git
 - C compiler -- gcc, clang...
 - Rust and Cargo -- https://rustup.rs/
 - `libindy` library on your `LD_LIBRARY_PATH`
   - https://github.com/hyperledger/indy-sdk#installing-the-sdk

2. Clone the code repository

3. From the `libmysqlstorage` directory, run the following command:

```
cargo build
```

After this you will have library artifacts in `libmysqlstorage/target/debug/` directory.

```
libmysqlstorage/target/debug/libmysqlstorage.d
libmysqlstorage/target/debug/libmysqlstorage.rlib
libmysqlstorage/target/debug/libmysqlstorage.so
```

## How to Test

### Test Environment Setup

Before starting tests you need to configure DB connection parameters.

This is done by setting the appropriate ENV variables in your execution environment, and is controlled by the code located in the [test_env](./libmysqlstorage/tests/test_utils/test_env.rs) file.

Variables that need to be set are:

```
DB_USER
DB_PASS
DB_WRITE_HOST
DB_READ_HOST
DB_PORT
DB_NAME
```

### Functional Tests

In order to run functional tests go to `libmysqlstorage` directory and run the following command:

```
cargo test
```

For more info take a look at [`libmysqlstorage/tests/api.rs`](./libmysqlstorage/tests/api.rs).

### Performance Tests

There are two different kinds  of performance tests:

#### Real World Performance Tests

These performance tests allows you to test the code against real world load.

For more info take a look at [`libmysqlstorage/tests/performance_tests.rs`](./libmysqlstorage/tests/performance_tests.rs).

In order to run these performance tests go to `libmysqlstorage` directory and run the following command:

```
cargo test --package mysqlstorage --test performance_tests performance -- --ignored
```

#### Baseline Performance Tests

These performance tests are useful for getting a general feel of performance/regression of API methods, but they are not testing effects of a real world load.

For more info take a look at [`libmysqlstorage/tests/chained_perf_test.rs`](./libmysqlstorage/tests/chained_perf_test.rs).

In order to run these performance tests go to `libmysqlstorage` directory and run the following command:

```
cargo test --package mysqlstorage --test chained_perf_test chaned_perf_test::perf_runner -- --exact --ignored
```

### Integration Tests

As this library depends on `libindy` we created Integration tests using Indy SDK Java Wrapper.

More information about these tests can be found in the integration tests [README.md](./libmysqlstorage/tests/java_libindy_integration_tests/README.md)


## How CI/CD pipelines work

Information on how CI/CD pipelines work can be found in the following [Readme.md](./devops/README.md)
