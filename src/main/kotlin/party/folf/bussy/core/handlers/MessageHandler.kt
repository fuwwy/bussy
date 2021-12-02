package party.folf.bussy.core.handlers

import kotlinx.coroutines.CoroutineName
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.async
import net.dv8tion.jda.api.events.message.guild.GuildMessageReceivedEvent
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import party.folf.bussy.Bussy

class MessageHandler(val bussy: Bussy) {
    private val logger: Logger = LoggerFactory.getLogger(MessageHandler::class.java)
    private val pressureHandler = PressureHandler()

    fun handle(event: GuildMessageReceivedEvent) {
        // TODO: cache guilds previously searched
        val job = bussy.async(NonCancellable + Dispatchers.IO + CoroutineName("Command Processor Guild Search")) {
            bussy.getGuild(event.guild.idLong)
        }

        job.invokeOnCompletion { error ->
            if (error != null) {
                logger.error("Failed to load guild ${event.guild.idLong}: $error")
                logger.trace(null, error)
                return@invokeOnCompletion
            }
            val guild = job.getCompleted()

            pressureHandler.handle(event, guild)
        }
    }
}