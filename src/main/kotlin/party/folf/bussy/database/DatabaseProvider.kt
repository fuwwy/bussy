package party.folf.bussy.database

import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import org.jetbrains.exposed.sql.Database
import org.jetbrains.exposed.sql.SchemaUtils
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction
import party.folf.bussy.Bussy
import party.folf.bussy.database.tables.BussyGuilds

class DatabaseProvider(private val bussy: Bussy) {

    lateinit var database: Database

    fun connect() {
        val config = bussy.config
        database = Database.connect(HikariDataSource(HikariConfig().apply {
            driverClassName = config.database.driver
            jdbcUrl = config.database.jdbcUrl
            username = config.database.user
            password = config.database.password
        }))
    }

    suspend fun runMigrations() {
        newSuspendedTransaction(db = database) {
            SchemaUtils.createMissingTablesAndColumns(BussyGuilds)
        }
    }
}