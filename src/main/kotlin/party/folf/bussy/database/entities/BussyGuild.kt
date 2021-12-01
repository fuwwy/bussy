package party.folf.bussy.database.entities

import org.jetbrains.exposed.dao.LongEntity
import org.jetbrains.exposed.dao.LongEntityClass
import org.jetbrains.exposed.dao.id.EntityID
import party.folf.bussy.database.tables.BussyGuilds

class BussyGuild(id: EntityID<Long>): LongEntity(id) {
    companion object : LongEntityClass<BussyGuild>(BussyGuilds)

    val guildId = this.id.value
    var moderationChannel by BussyGuilds.moderationChannel
}