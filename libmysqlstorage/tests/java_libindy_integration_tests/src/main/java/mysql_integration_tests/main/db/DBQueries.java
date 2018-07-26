package mysql_integration_tests.main.db;

import java.sql.SQLException;

public class DBQueries {

    private static DBConnection dbConn;

    private static String DELETE_ALL_QUERY = "DELETE FROM wallets";

    public static void setDBConnection(DBConnection conn){
        dbConn = conn;
    }

    public static void deleteAll() throws SQLException {
        dbConn.executeQuery(DELETE_ALL_QUERY);
    }
}
