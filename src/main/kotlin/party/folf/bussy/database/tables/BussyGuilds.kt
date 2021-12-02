package party.folf.bussy.database.tables

import org.jetbrains.exposed.dao.id.LongIdTable

object BussyGuilds : LongIdTable() {
    val moderationChannel = long("moderationChannel").nullable()
    val raidContainmentChannel = long("raidContainmentChannel").nullable()
    val silenceContainmentChannel = long("silenceContainmentChannel").nullable()
    val logChannel = long("logChannel").nullable()

    val memberRole = long("memberRole").nullable()
    val silenceRole = long("silenceRole").nullable()
    val newRole = long("newRole").nullable()
    val roleSetupComplete = bool("roleSetupComplete").default(false)

    val maxPressure = double("maxPressure").default(60.0)
    val basePressure = double("basePressure").default(10.0)
    val mediaPressure = double("mediaPressure").default(8.3)
    val charPressure = double("charPressure").default(0.00625)
    val linePressure = double("linePressure").default(0.714)
    val mentionPressure = double("mentionPressure").default(2.5)
    val repeatPressure = double("repeatPressure").default(10.0)
    val pressureDropoff = double("pressureDropoff").default(2.0)
}