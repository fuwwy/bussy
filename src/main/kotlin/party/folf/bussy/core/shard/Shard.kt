package party.folf.bussy.core.shard

import net.dv8tion.jda.api.JDA

class Shard(val id: Int) {

    internal lateinit var jda: JDA

    val eventListener = ShardListener(this)

}