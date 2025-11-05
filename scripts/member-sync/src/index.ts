import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { Client, Events, GatewayIntentBits } from 'discord.js'
import { parse } from 'smol-toml'
import { difference } from 'es-toolkit'

type Identifiable = { id: string }

const studyDataPath = resolve(import.meta.dirname, '../../../study.toml')
const studyData = (await readFile(studyDataPath, 'utf-8').then(parse)) as {
	lecturer: Identifiable
	members: Array<Identifiable>
}

const ids = [studyData.lecturer.id, ...studyData.members.map(({ id }) => id)]

const client: Client<true> = new Client({
	intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildMembers],
})

client.login(process.env.DISCORD_TOKEN)

await new Promise(r => client.once(Events.ClientReady, r))

const guild = await client.guilds.fetch(process.env.GUILD_ID!)

await guild.members.fetch()

const role = await guild.roles.fetch(process.env.ROLE_ID!)
if (!role) throw new Error(`Role with ID ${process.env.ROLE_ID} not found.`)

const unlistedUsers = difference(
	role.members.map(({ id }) => id),
	ids
)

await Promise.all([
	...ids.map(async (id, i) => {
		const member = await guild.members.fetch(id)

		if (member.roles.cache.has(role.id)) return

		await member.roles
			.add(role)
			.then(() => console.log(`+ ${member.user.tag}`))
			.catch(console.error)
	}),
	...unlistedUsers.map(async id => {
		const member = await guild.members.fetch(id)

		await member.roles
			.remove(role)
			.then(() => console.log(`- ${member.user.tag}`))
			.catch(console.error)
	}),
])

await client.destroy()
