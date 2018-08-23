package mysql_integration_tests.main.db;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.SQLException;

public class DBConnection {

    private static Logger logger = LoggerFactory.getLogger(DBConnection.class);

    private static Connection conn;

    public DBConnection(String connectionString, String username, String pwd) throws SQLException, ClassNotFoundException, IllegalAccessException, InstantiationException {
        if (conn == null) {
            Class.forName("com.mysql.jdbc.Driver");
            conn = DriverManager
                    .getConnection(connectionString, username, pwd);
        }
    }

    public boolean execute(String query) throws SQLException {
        logger.debug("executing query: '" + query + "'");
        return conn.createStatement().execute(query);
    }

    public void close() throws SQLException {
        if(!conn.isClosed()) conn.close();
    }
}
