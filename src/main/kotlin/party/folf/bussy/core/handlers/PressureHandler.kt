package party.folf.bussy.core.handlers

import net.dv8tion.jda.api.events.message.guild.GuildMessageReceivedEvent
import party.folf.bussy.database.entities.BussyGuild
import java.util.concurrent.ConcurrentHashMap

class PressureHandler {
    companion object {
        private val pressureUsers: MutableMap<String, PressureUser> = ConcurrentHashMap()
    }

    fun handle(event: GuildMessageReceivedEvent, guild: BussyGuild) {
        val user = pressureUsers.computeIfAbsent("${guild.guildId}#${event.author.id}") { PressureUser() }

        val lastMessage = user.lastMessageTimestamp
        user.lastMessageTimestamp = System.currentTimeMillis()
        val interval = user.lastMessageTimestamp - lastMessage
        if (interval < user.lastMessageTimestamp) {
            user.pressure -= guild.pressureDropoff * (interval / 1000)
        }
        if (user.pressure < 0) user.pressure = 0.0

        user.pressure += guild.basePressure
        user.pressure += guild.mediaPressure * event.message.attachments.size
        user.pressure += guild.mediaPressure * event.message.embeds.size
        user.pressure += guild.charPressure * event.message.contentDisplay.length
        user.pressure += guild.linePressure * event.message.contentRaw.count { it == '\n' }
        user.pressure += guild.mentionPressure * event.message.mentionedUsers.size
        user.pressure += guild.mentionPressure * event.message.mentionedRoles.size
        if (event.message.contentStripped.isNotEmpty()) {
            if (event.message.contentStripped == user.lastMessage) user.pressure += guild.repeatPressure
            user.lastMessage = event.message.contentStripped
        }

        if (user.pressure > guild.maxPressure) {
            event.channel.sendMessage("Silence bitch " + user.pressure).queue()
            user.pressure = 0.0
        }
    }

    class PressureUser(
        var pressure: Double = 0.0,
        var lastMessageTimestamp: Long = 0,
        var lastMessage: String? = null
    )
}