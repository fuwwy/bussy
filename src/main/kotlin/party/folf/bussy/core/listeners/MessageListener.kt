package party.folf.bussy.core.listeners

import net.dv8tion.jda.api.events.GenericEvent
import net.dv8tion.jda.api.events.message.guild.GuildMessageReceivedEvent
import net.dv8tion.jda.api.hooks.EventListener
import party.folf.bussy.core.handlers.MessageHandler

class MessageListener(
    private val handler: MessageHandler
): EventListener {

    override fun onEvent(event: GenericEvent) {
        if (event !is GuildMessageReceivedEvent)
            return

        if (event.author.isBot || event.isWebhookMessage)
            return

        try {
            handler.handle(event)
        } catch (e: Throwable) {
            // TODO Exception handling
            e.printStackTrace()
        }
    }
}