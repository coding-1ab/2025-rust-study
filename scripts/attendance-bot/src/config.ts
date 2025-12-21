import { z } from 'zod'

export const envSchema = z.object({
	DISCORD_TOKEN: z.string().min(1),
	APPLICATION_ID: z.string().min(1),
	GUILD_ID: z.string().min(1),
	TEXT_CHANNEL_ID: z.string().min(1),
	VOICE_CHANNEL_IDS: z.string().min(1), // comma-separated

	// doesn't work for now.
	TIMEZONE: z.string().optional().default('UTC'),
})

const parsed = envSchema.safeParse(process.env)
if (!parsed.success) {
	console.error('Invalid configuration:', parsed.error.flatten().fieldErrors)
	process.exit(1)
}

export const config = {
	token: parsed.data.DISCORD_TOKEN,
	applicationId: parsed.data.APPLICATION_ID,
	guildId: parsed.data.GUILD_ID,
	textChannelId: parsed.data.TEXT_CHANNEL_ID,
	voiceChannelIds: parsed.data.VOICE_CHANNEL_IDS.split(',')
		.map((s: string) => s.trim())
		.filter(Boolean),
	timezone: parsed.data.TIMEZONE,
}
