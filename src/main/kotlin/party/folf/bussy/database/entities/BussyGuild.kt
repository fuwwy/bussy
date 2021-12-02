package party.folf.bussy.database.entities

import org.jetbrains.exposed.dao.LongEntity
import org.jetbrains.exposed.dao.LongEntityClass
import org.jetbrains.exposed.dao.id.EntityID
import party.folf.bussy.database.tables.BussyGuilds

class BussyGuild(id: EntityID<Long>) : LongEntity(id) {
    companion object : LongEntityClass<BussyGuild>(BussyGuilds)

    val guildId = this.id.value
    var moderationChannel by BussyGuilds.moderationChannel
    var raidContainmentChannel by BussyGuilds.raidContainmentChannel
    var silenceContainmentChannel by BussyGuilds.silenceContainmentChannel
    var logChannel by BussyGuilds.logChannel

    var memberRole by BussyGuilds.memberRole
    var silenceRole by BussyGuilds.silenceRole
    var newRole by BussyGuilds.newRole
    var roleSetupComplete by BussyGuilds.roleSetupComplete
}