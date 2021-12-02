package party.folf.bussy.core.handlers

import net.dv8tion.jda.api.events.message.guild.GuildMessageReceivedEvent
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import party.folf.bussy.database.entities.BussyGuild
import java.util.concurrent.ConcurrentHashMap

class PressureHandler {

    companion object {
        private val pressureUsers: MutableMap<String, PressureUser> = ConcurrentHashMap()
    }

    fun handle(event: GuildMessageReceivedEvent, guild: BussyGuild) {
        val user = pressureUsers.computeIfAbsent("${guild.guildId}#${event.author.id}") { PressureUser(event.author.idLong) }

        val lastMessage = user.lastMessageTimestamp
        user.lastMessageTimestamp = System.currentTimeMillis()
        val interval = user.lastMessageTimestamp - lastMessage
        if (interval < user.lastMessageTimestamp) {
            user.pressure -= guild.pressureDropoff * (interval / 1000)
        }
        if (user.pressure < 0) user.pressure = 0.0

        run {
            if (user.addPressure(guild, guild.basePressure)) return@run
            if (user.addPressure(guild, guild.mediaPressure * event.message.attachments.size)) return@run
            if (user.addPressure(guild, guild.mediaPressure * event.message.embeds.size)) return@run
            if (user.addPressure(guild, guild.charPressure * event.message.contentDisplay.length)) return@run
            if (user.addPressure(guild, guild.linePressure * event.message.contentRaw.count { it == '\n' })) return@run
            if (user.addPressure(guild, guild.mentionPressure * event.message.mentionedUsers.size)) return@run
            if (user.addPressure(guild, guild.mentionPressure * event.message.mentionedRoles.size)) return@run
        }

        if (event.message.contentStripped.isNotEmpty()) {
            if (event.message.contentStripped == user.lastMessage) user.pressure += guild.repeatPressure
            user.lastMessage = event.message.contentStripped
        }

        if (user.pressure > guild.maxPressure) {
            PressureUser.logger.trace("User ${guild.guildId}#${event.author.id} surpassed max pressure. (${guild.maxPressure})")
            event.channel.sendMessage("Silence bitch " + user.pressure).queue()
            user.pressure = 0.0
        }
    }

    class PressureUser(
        var userId: Long,
        var pressure: Double = 0.0,
        var lastMessageTimestamp: Long = 0,
        var lastMessage: String? = null
    ) {
        companion object {
            val logger: Logger = LoggerFactory.getLogger(PressureHandler::class.java)
        }

        fun addPressure(guild: BussyGuild, pressure: Double): Boolean {
            logger.trace("Adding $pressure pressure to user $userId on guild ${guild.guildId}")
            this.pressure += pressure
            return pressure > guild.maxPressure
        }
    }
}