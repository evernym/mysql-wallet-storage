# Prerequisites

Apache maven and Java JRE 1.8 installed on the machine where tests will be executed <br>
Copy
    - `libindy.so` and `libaurorastorage.so` for Linux
    - `libindy.dll` and `libaurorastorage.dll` for Windows
    - `libindy.dylib` and `libaurorastorage.dylib` for Mac
to `lib` subfolder located in the same folder as this Readme file.

# Tests executon

In order to run Libindy - Aurora storage *integration tests* execute this from the command line
```
mvn clean versions:update-properties validate test -DsuiteFile=NonSecrets-PositiveTests.xml
```
or
```
mvn clean versions:update-properties validate test -DsuiteFile=NonSecrets-NegativeTests.xml
```

In order to run Libindy - Aurora storage *longevity tests* execute this from the command line
```
mvn clean versions:update-properties validate test -DsuiteFile=AuroraStorageLongevityTest.xml
```