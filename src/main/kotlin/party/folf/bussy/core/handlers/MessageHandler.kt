package party.folf.bussy.core.handlers

import kotlinx.coroutines.*
import net.dv8tion.jda.api.events.message.guild.GuildMessageReceivedEvent
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import party.folf.bussy.Bussy

class MessageHandler(val bussy: Bussy) {
    private val log: Logger = LoggerFactory.getLogger(MessageHandler::class.java)

    fun handle(event: GuildMessageReceivedEvent) {
        // TODO: cache guilds previously searched
        val job = bussy.async(NonCancellable + Dispatchers.IO + CoroutineName("Command Processor Guild Search")) {
            bussy.getGuild(event.guild.idLong)
        }

        job.invokeOnCompletion { error ->
            if (error != null) {
                log.error("Failed to load guild ${event.guild.idLong}: $error")
                log.trace(null, error)
                return@invokeOnCompletion
            }
            val guild = job.getCompleted()

            event.channel.sendMessage(guild.guildId.toString()).queue()
        }
    }
}