import { TZDate } from '@date-fns/tz'

// yyyy-mm-dd
export type Datekey = string

export const tz = 'Asia/Seoul'

const msPerDay = 24 * 60 * 60 * 1000

export const dateToDatekey = (date = new Date()) => {
	const fmt = new Intl.DateTimeFormat('en-CA', {
		timeZone: tz,
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
	})
	const parts = fmt.formatToParts(date)

	const y = parts.find(p => p.type === 'year')?.value ?? '0000'
	const m = parts.find(p => p.type === 'month')?.value ?? '00'
	const d = parts.find(p => p.type === 'day')?.value ?? '00'

	return `${y}-${m}-${d}`
}

export const datekeyToDate = (dateKey: Datekey): Date => {
	const [year, month, day] = dateKey.split('-').map(Number)
	return new Date(year!, month! - 1, day)
}

export const datekeyWeekday = (dateKey: Datekey) =>
	datekeyToDate(dateKey)
		.toLocaleDateString('en-US', {
			weekday: 'long',
			timeZone: tz,
		})
		.toLowerCase()

// study start date will be provided as an offset.
// the entire study is weekly basis, with the first study recorded under Week 1.
// e.g., Week 1 [ study 1, 2, 3 ], Week 2 [ study 4, 5 ], Week 3 [ study 6, 7, 8 ], ...
// find the week name based on the given specific study date.
// note: week depends on the weekday. if the study starts on sunday, the successive date will be under Week 2.
// i.e., naive modulo 7 does not work here.
// (week starts with Monday.)
export const dateToStudyWeek = (startDay: Date, date: Date): string => {
	// indicates how many days should be shifted to align week start.
	const startDayWeekShift = (startDay.getUTCDay() + 6) % 7 // monday starts at 0

	const daysDiff = Math.floor(
		(date.getTime() - startDay.getTime()) / msPerDay
	)
	console.log({ daysDiff, startDayWeekShift })

	const alignedDiff = startDayWeekShift + daysDiff

	const weekNumber = Math.floor(alignedDiff / 7) + 1

	return `Week ${weekNumber}`
}

function formatTable(
	records: { userId: string; name: string; timestamp: string }[]
): string {
	if (records.length === 0) return 'No one has checked in yet.'
	const rows = records
		.sort((a, b) => a.timestamp.localeCompare(b.timestamp))
		.map((r, i) => {
			const time = new Date(r.timestamp).toLocaleTimeString(undefined, {
				hour: '2-digit',
				minute: '2-digit',
			})
			return `${String(i + 1).padStart(2, ' ')}. ${r.name} (${
				r.userId
			}) - ${time}`
		})
	return '```\n' + rows.join('\n') + '\n```'
}

// check whether the current time is study time or not.
// use the studyData information.

export const isInTimeRange = (start: Date, end: Date, now: Date) => {
	// Constants
	const MS_PER_DAY = 24 * 60 * 60 * 1000
	// Asia/Seoul is UTC+9 — interpret TOML time literals as hh:mm:ss in this TZ.
	const TZ_OFFSET_HOURS = 9
	const TZ_OFFSET_MS = TZ_OFFSET_HOURS * 60 * 60 * 1000

	// safe modulo into [0, MS_PER_DAY)
	const modDay = (ms: number) => ((ms % MS_PER_DAY) + MS_PER_DAY) % MS_PER_DAY

	// TomlDate: its UTC components represent the literal hh:mm:ss from TOML.
	// So use UTC getters directly (do NOT apply timezone shift to start/end).
	const msOfTomlTime = (d: Date) =>
		modDay(
			d.getUTCHours() * 3600_000 +
				d.getUTCMinutes() * 60_000 +
				d.getUTCSeconds() * 1000 +
				d.getUTCMilliseconds()
		)

	// For "now", compute the time-of-day in the target timezone (Asia/Seoul).
	// Take now's UTC ms-of-day and add TZ offset, then normalize.
	const msOfNowInTZ = (d: Date) =>
		modDay(
			d.getUTCHours() * 3600_000 +
				d.getUTCMinutes() * 60_000 +
				d.getUTCSeconds() * 1000 +
				d.getUTCMilliseconds() +
				TZ_OFFSET_MS
		)

	const s = msOfTomlTime(start)
	const e = msOfTomlTime(end)
	const n = msOfNowInTZ(now)

	// If start === end (after normalization), treat as full-day range (always true).
	if (s === e) return true

	// Non-cross-midnight range: start <= now <= end
	if (s < e) {
		return n >= s && n <= e
	}

	// Cross-midnight range: now >= start OR now <= end
	return n >= s || n <= e
}

export const weekdayKeyArr = [
	'sunday',
	'monday',
	'tuesday',
	'wednesday',
	'thursday',
	'friday',
	'saturday',
] as const

export type WeekdayKey = (typeof weekdayKeyArr)[number]

export const weekdayKey = (date: Date) =>
	weekdayKeyArr[new TZDate(date, tz).getDay()]

export const koreanWeekdayName = (weekdayKey: WeekdayKey) =>
	({
		monday: '월요일',
		tuesday: '화요일',
		wednesday: '수요일',
		thursday: '목요일',
		friday: '금요일',
		saturday: '토요일',
		sunday: '일요일',
	}[weekdayKey])
