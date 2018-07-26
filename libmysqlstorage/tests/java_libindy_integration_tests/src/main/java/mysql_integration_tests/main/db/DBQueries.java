package mysql_integration_tests.main.db;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.sql.SQLException;

public class DBQueries {

    private static DBConnection dbConn;

    private static String DELETE_ALL_QUERY = "DELETE FROM wallets";
    private static Logger logger = LoggerFactory.getLogger(DBConnection.class);

    public static void setDBConnection(DBConnection conn){
        dbConn = conn;
    }

    public static void deleteAll() throws SQLException {
        dbConn.execute(DELETE_ALL_QUERY);
        logger.debug("Deleting all from db...");
    }
}
