package party.folf.bussy

import com.google.common.util.concurrent.ThreadFactoryBuilder
import kotlinx.coroutines.CompletableDeferred
import net.dv8tion.jda.api.entities.Activity
import net.dv8tion.jda.api.entities.Message
import net.dv8tion.jda.api.events.GenericEvent
import net.dv8tion.jda.api.events.ReadyEvent
import net.dv8tion.jda.api.hooks.EventListener
import net.dv8tion.jda.api.requests.GatewayIntent
import net.dv8tion.jda.api.requests.restaction.MessageAction
import net.dv8tion.jda.api.sharding.DefaultShardManagerBuilder
import net.dv8tion.jda.api.sharding.ShardManager
import net.dv8tion.jda.api.utils.ChunkingFilter
import net.dv8tion.jda.api.utils.cache.CacheFlag
import org.slf4j.Logger
import party.folf.bussy.core.shard.Shard
import party.folf.bussy.database.DatabaseProvider
import java.time.Duration
import java.time.Instant
import java.util.*
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicInteger
import java.util.function.IntFunction
import java.util.stream.Collectors
import kotlin.time.ExperimentalTime

class Bussy(val config: BussyConfig) {
    private val log: Logger = logger<Bussy>()

    lateinit var databaseProvider: DatabaseProvider
    private val shards: ConcurrentHashMap<Int, Shard> = ConcurrentHashMap()
    lateinit var shardManager: ShardManager

    suspend fun start() {
        log.info("Connecting to the database...")
        databaseProvider = DatabaseProvider(this)
        databaseProvider.connect()
        databaseProvider.runMigrations()

        startShards()
    }

    private suspend fun startShards() {
        val defaultDeniedMentions =
            EnumSet.of(Message.MentionType.EVERYONE, Message.MentionType.HERE, Message.MentionType.ROLE)
        MessageAction.setDefaultMentions(EnumSet.complementOf(defaultDeniedMentions))

        val enabledGatewayIntents = listOf(
            GatewayIntent.GUILD_MESSAGES,
            GatewayIntent.GUILD_MESSAGE_REACTIONS,
            GatewayIntent.GUILD_MEMBERS
        )

        log.info("Using intents {}", enabledGatewayIntents
            .stream()
            .map { it.name }
            .collect(Collectors.joining(", "))
        )

        val latchCount = config.shardCount
        val shardCounter = AtomicInteger(0)
        val shardReadyState = CompletableDeferred<Unit>()
        val shardListener = object: EventListener {
            override fun onEvent(event: GenericEvent) {
                if (event !is ReadyEvent)
                    return

                val value = shardCounter.incrementAndGet()
                log.info("Shard $value of $latchCount loaded.")

                if (value == latchCount)
                    shardReadyState.complete(Unit)
            }
        }

        val shardManagerBuilder = DefaultShardManagerBuilder.create(config.token, enabledGatewayIntents)
            .setChunkingFilter(ChunkingFilter.NONE)
            .addEventListeners(shardListener)
            .addEventListenerProviders(
                // event listener providers (commands, members, etc) go here
                listOf(
                    IntFunction { shardId -> getShard(shardId).eventListener }
                ))
            .setBulkDeleteSplittingEnabled(false)
            .disableCache(EnumSet.of(CacheFlag.ACTIVITY, CacheFlag.EMOTE, CacheFlag.CLIENT_STATUS))
            .setActivity(Activity.playing("Bussy is loading..."))

        shardManagerBuilder.setShardsTotal(latchCount)
            .setGatewayPool(Executors.newSingleThreadScheduledExecutor(ThreadFactoryBuilder()
                .setNameFormat("GatewayThread-%d")
                .setDaemon(true)
                .setPriority(Thread.MAX_PRIORITY)
                .build()), true)
            .setRateLimitPool(Executors.newScheduledThreadPool(2, ThreadFactoryBuilder()
                .setNameFormat("RequesterThread-%d")
                .setDaemon(true)
                .build()), true)

        log.info("Spawning $latchCount shards...")
        shardManager = shardManagerBuilder.build()

        val startedAt = Instant.now()
        shardReadyState.invokeOnCompletion {
            shardManager.removeEventListener(shardListener)
            val duration = Duration.between(startedAt, Instant.now()).toSeconds()
            log.info("Loaded all shards successfully! Took $duration seconds.")
        }
    }

    fun getShard(id: Int): Shard {
        return shards.computeIfAbsent(id) { Shard(id) }
    }
}