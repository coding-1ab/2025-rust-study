import {
	Client,
	GatewayIntentBits,
	Partials,
	REST,
	Routes,
	TextChannel,
	PermissionFlagsBits,
	EmbedBuilder,
	type Interaction,
	VoiceState,
	Events,
} from 'discord.js'
import { config } from './config'
import { AttendanceStore, MarkResult } from './attendance-store'
import type { StudyData } from './study'
import { CommandNames } from './commands'
import {
	isInTimeRange,
	koreanWeekdayName,
	weekdayKey,
	type WeekdayKey,
} from './utils'
import { difference } from 'es-toolkit'
import { schedule } from 'node-cron'

export class Bot extends Client {
	#config = config
	#studyData: StudyData
	#store: AttendanceStore

	constructor(studyData: StudyData, store: AttendanceStore) {
		super({
			intents: [
				GatewayIntentBits.Guilds,
				GatewayIntentBits.GuildVoiceStates,
				GatewayIntentBits.GuildMembers,
			],
			partials: [Partials.GuildMember, Partials.User],
		})

		this.#studyData = studyData
		this.#store = store

		this.once(Events.ClientReady, async client => {
			console.log(`Logged in as ${client.user?.tag}`)

			// register slash commands
			try {
				await this.#registerCommands()

				console.log('Slash commands registered.')
			} catch (e) {
				console.error('Failed to register commands', e)
			}

			// setup scheduled notifications (10 minutes before each study session)
			try {
				await this.#setupStudyNotifications()
				console.log('Study notifications scheduled.')
			} catch (e) {
				console.error('Failed to schedule study notifications', e)
			}
		})

		this.on(Events.InteractionCreate, async (interaction: Interaction) => {
			if (!interaction.isChatInputCommand()) return

			const cmd = interaction.commandName

			if (cmd === CommandNames.MEMBERS) {
				const saturdayMembers = this.#studyData.members.filter(
					m => m.preferred_study_weekday === 'saturday'
				)
				const sundayMembers = this.#studyData.members.filter(
					m => m.preferred_study_weekday === 'sunday'
				)

				// list all members from study data
				const saturdayList = saturdayMembers.map(m => ` 1. <@${m.id}>`)

				const sundayList = sundayMembers.map(m => ` 1. <@${m.id}>`)

				await interaction.reply({
					embeds: [
						new EmbedBuilder()
							.setTitle('<2025년 연말 Rust 스터디> 스터디원')
							.addFields(
								{
									name: '토요일반',
									value:
										saturdayList.join('\n') || 'No members',
									inline: false,
								},
								{
									name: '일요일반',
									value:
										sundayList.join('\n') || 'No members',
									inline: false,
								}
							)
							.setColor(0x5865f2),
					],
					ephemeral: false,
				})
			} else if (cmd === CommandNames.ATTENDANCE) {
				// show today's attendance

				const day = await this.#store.getToday()
				const todayWeekday = weekdayKey(new Date())

				const supposedToAttendToday = this.#studyData.members.filter(
					m => m.preferred_study_weekday === todayWeekday
				)

				const supposedToAttendTodayLines = supposedToAttendToday.map(
					m =>
						day.attended.includes(m.id)
							? ` 1. :white_check_mark: <@${m.id}>`
							: ` 1. :x: <@${m.id}>`
				)

				const rest = difference(
					day.attended,
					supposedToAttendToday.map(m => m.id)
				).map(id => ` * <@${id}>`)

				await interaction.reply({
					embeds: [
						new EmbedBuilder()
							.setTitle(
								`출석 현황 - ${day.date}(${koreanWeekdayName(
									day.weekday as WeekdayKey
								)})`
							)
							.setDescription(
								`${supposedToAttendTodayLines.join(
									'\n'
								)}\n\n정원 외 출석:\n${
									rest.join('\n').trim() || '(없음)'
								}`
							)
							.setColor(0x00b894),
					],
					ephemeral: false,
				})
			}
			// else if (cmd === CommandNames.ATTENDANCE_STATS) {
			// 	// not implemented yet
			// 	await interaction.reply({
			// 		content: 'Attendance stats not implemented yet.',
			// 		ephemeral: true,
			// 	})
			// }
			// else if (cmd === CommandNames.ATTENDANCE_RESET) {
			// 	if (
			// 		!interaction.memberPermissions?.has(
			// 			PermissionFlagsBits.ManageGuild
			// 		)
			// 	) {
			// 		await interaction.reply({
			// 			content:
			// 				'You need Manage Server permission to use this.',
			// 			ephemeral: true,
			// 		})
			// 		return
			// 	}
			// 	await this.#store.resetToday()
			// 	await interaction.reply({
			// 		content: "Today's attendance has been reset.",
			// 		ephemeral: false,
			// 	})
			// }
		})

		this.on(
			Events.VoiceStateUpdate,
			async (oldState: VoiceState, newState: VoiceState) => {
				const joinedChannelId = newState.channelId
				const leftChannelId = oldState.channelId

				if (joinedChannelId === leftChannelId) return
				if (!joinedChannelId) return
				if (!config.voiceChannelIds.includes(joinedChannelId)) return

				const member = newState.member || oldState.member
				if (!member) return

				const displayName = member.displayName || member.user.username
				const markResult = await store.mark(member.id)

				console.log(
					`User ${displayName} (${member.id}) joined ${joinedChannelId}. ${markResult}`
				)

				if (markResult === MarkResult.Marked) {
					const todayCount = (await this.#store.todayEntry()).attended
						.length

					const embed = new EmbedBuilder()
						.setTitle('출석 완료!')
						.setDescription(
							`<@${member.id}> 님이 출석하셨습니다. 🎉`
						)
						.setColor(0x00b894)
						.setTimestamp(new Date())
						.setFooter({
							text: `오늘 ${todayCount}번째 출석자입니다.`,
						})

					await (newState.channel as unknown as TextChannel).send({
						embeds: [embed],
					})
				} else if (markResult === MarkResult.NotAMember) {
					const embed = new EmbedBuilder()
						.setTitle('환영합니다!')
						.setDescription(`<@${member.id}> 님이 방문하셨어요!`)
						.setColor(0x2e59d7)
						.setTimestamp(new Date())

					await (newState.channel as unknown as TextChannel).send({
						embeds: [embed],
					})
				}
			}
		)
	}

	async #registerCommands() {
		const commands = [
			{
				name: CommandNames.ATTENDANCE,
				description: '오늘 출석해야 하는 사람과 한 사람을 보여줍니다.',
			},
			{
				name: CommandNames.MEMBERS,
				description: '스터디원을 보여줍니다.',
			},
			// {
			// 	name: CommandNames.ATTENDANCE_STATS,
			// 	description:
			// 		'Show overall attendance stats (not implemented yet)',
			// },
			// {
			// 	name: CommandNames.ATTENDANCE_RESET,
			// 	description: "Reset today's attendance (Manage Server only)",
			// 	default_member_permissions: String(
			// 		PermissionFlagsBits.ManageGuild
			// 	),
			// },
		]

		const rest = new REST({ version: '10' }).setToken(config.token)
		await rest.put(
			Routes.applicationGuildCommands(
				config.applicationId,
				config.guildId
			),
			{ body: commands }
		)
	}

	// New: schedule notifications based on studyData.study_times
	async #setupStudyNotifications() {
		const guild = this.guilds.cache.get(this.#config.guildId)
		if (!guild) {
			console.warn('Guild not found, skipping study notifications.')
			return
		}

		// TODO: this is hardcoded for now
		const crons = [`50 18 * * 6`, `50 19 * * 0`]

		for (const cronExpr of crons) {
			schedule(
				cronExpr,
				async () => {
					console.log('plz')

					try {
						const ch = await guild.channels.fetch(
							config.textChannelId
						)
						if (!ch || !ch.isTextBased()) {
							console.error(
								'Text channel not found or not text-based for study notification.'
							)
							return
						}

						const embed = new EmbedBuilder()
							.setTitle('스터디 알림')
							.setDescription(
								`스터디가 10분 후에 시작해요! 다들 <#${config.voiceChannelIds}>에 참여해 주세요!`
							)
							.setColor(0x00b894)
							.setTimestamp(new Date())

						await ch.send({ embeds: [embed] })
					} catch (err) {
						console.error('Failed to send study notification:', err)
					}
				},
				{
					timezone: 'Asia/Seoul',
				}
			)

			console.log(`Scheduled study notification for ${cronExpr})`)
		}
	}

	override async login() {
		return super.login(this.#config.token)
	}
}
