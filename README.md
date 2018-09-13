# Indy SDK MySQL Wallet Storage Plug-In

## About

Although Indy infrastructure provides only one wallet implementation (sqlite based), it allows developers to plug different storage backends.

This project is an implementation of such a plug-in for MySQL Storage Backend. By using it you can switch to any MySQL compliant database for your Indy SDK use cases.

## Dependencies

 - Git
 - C compiler -- gcc, clang...
 - Rust and Cargo -- https://rustup.rs/
 - `libindy` library on your `LD_LIBRARY_PATH` -- https://github.com/hyperledger/indy-sdk#installing-the-sdk

## How to Build

After cloning the repo go to `libmysqlstorage` directory and run the following command:

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

### Test Config

Before starting tests you need to configure DB connection parameters. This is done in [test config](libmysqlstorage/tests/test_utils/test_env.rs) file.

This file contains different sections ie. Config Types for different environments. Config type is read from a environment variable `WALLET_CONFIG_TYPE`, but if this variable is not set tests will run with the `DEV` Config Type.

```
ConfigType::DEV => {
                Config {
                    config: json!(
                        {
                            "read_host": "localhost",
                            "write_host": "localhost",
                            "port": 3306,
                            "db_name": "wallet"
                        }
                    ).to_string(),
                    credentials: json!(
                        {
                            "user": "wallet",
                            "pass": "wallet"
                        }
                    ).to_string()
                }
            }
```

### Functional Tests

In order to run functional tests go to `libmysqlstorage` directory and run the following command:

```
cargo test --package mysqlstorage --test api high_casees
```

### Performance Tests

There are two different kinds  of performance tests:

#### Real World Performance Tests

These performance tests allows you to test the code against real world load.

For more info take a look at [`libmysqlstorage/tests/performance_tests.rs`](./libmysqlstorage/tests/performance_tests.rs).

In order to run these performance tests go to `libmysqlstorage` directory and run the following command:

```
cargo test --package mysqlstorage --test performance_tests performance
```

#### Baseline Performance Tests

These performance tests are useful for getting a general feel of performance/regression of API methods, but they are not testing effects of a real world load.

For more info take a look at [`libmysqlstorage/tests/chained_perf_test.rs`](./libmysqlstorage/tests/chained_perf_test.rs).

In order to run these performance tests go to `libmysqlstorage` directory and run the following command:

```
cargo test --package aurorastorage --test chained_perf_test chaned_perf_test::perf_runner -- --exact
```

### Integration Tests

As this library depends on `libindy` we created Integration tests using Indy SDK Java Wrapper.

More information about these tests can be found in the integration tests [README.md](./libmysqlstorage/tests/java_libindy_integration_tests/README.md)

## How to Use

In order to use MySQL storage plugin you need to call `mysql_storage_init` function from your code.

This will register the plug-in to `libindy` and allow you to redirect your requests to MySQL Storage by using `"mysql"` wallet type.

