In order to run tests execute this from the command line
```
mvn clean versions:update-properties validate test -DsuiteFile=NonSecrets-PositiveTests.xml
```
or
```
mvn clean versions:update-properties validate test -DsuiteFile=NonSecrets-NegativeTests.xml
```