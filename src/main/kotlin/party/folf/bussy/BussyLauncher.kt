package party.folf.bussy

import com.sksamuel.hoplite.ConfigLoader
import kotlinx.coroutines.runBlocking
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import party.folf.bussy.core.LogFilter
import java.io.File
import kotlin.system.exitProcess

object BussyLauncher {
    private val config: BussyConfig by lazy { loadConfig() }
    private val log: Logger = logger<BussyLauncher>()
    lateinit var bussy: Bussy

    @JvmStatic
    fun main(args: Array<String>) {
        runBlocking {
            log.info("Starting Bussy...")
            log.info("Filtering all logs below {}", LogFilter.LEVEL)

            bussy = Bussy(config)
            bussy.start()

            log.info("Finished loading.")
        }
    }

    private fun loadConfig(): BussyConfig {
        return runCatching {
            ConfigLoader().loadConfigOrThrow<BussyConfig>(File("config.json"))
        }.onFailure { error ->
            log.error("Failed to load config: $error")
            log.trace(null, error)
            exitProcess(1)
        }.getOrThrow()
    }
}

inline fun <reified T> logger(): Logger = LoggerFactory.getLogger(T::class.java)