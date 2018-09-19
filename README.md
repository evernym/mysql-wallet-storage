# Indy SDK MySQL Wallet Storage Plug-In

## About

Indy SDK provides only one wallet implementation (sqlite based), but it allows developers to plug different storage backends.

This project is an implementation of such a plug-in for MySQL Storage Backend. By using it you can switch to any MySQL compliant database for your Indy SDK use cases.

## Supported platforms

At this moment only 64-bit Ubuntu LTS 16.04 (Xenial) is supported.

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
    </tr>
    <tr>
      <td rowspan="2">Ubuntu</td>
      <td> 16.04 LTS (Xenial Xerus) </td>
      <td> &#10006; </td>
      <td> &#10004; </td>
    </tr>
    <tr>
      <td> 18.04 LTS (Bionic Beaver) </td>
      <td> &#10006; </td>
      <td> &#10006; </td>
    </tr>
  </tbody>
</table>

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

### Test Environment Setup

Before starting tests you need to configure DB connection parameters.

This is done by setting approriate ENV variables in your execution environment, and is controlled by the code located 
in the [test_env](libmysqlstorage/tests/test_utils/test_env.rs) file.

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

## How to Use

In order to use MySQL storage plugin you need to call `mysql_storage_init` function from your code.

This will register the plug-in to `libindy` and allow you to redirect your requests to MySQL Storage by using `"mysql"` wallet type.

## How CI/CD pipelines work

Information on how CI/CD pipelines work can be found in the following [Readme.md](./devops/README.md)
