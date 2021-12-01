package party.folf.bussy.database.tables

import org.jetbrains.exposed.dao.id.LongIdTable

object BussyGuilds : LongIdTable() {
    val moderationChannel = long("moderationChannel").nullable()
}