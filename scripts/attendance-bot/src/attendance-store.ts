import { readFile, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { parse, stringify } from 'smol-toml'
import { datekeyWeekday, dateToDatekey, dateToStudyWeek } from './utils'
import type { StudyData } from './study'

const dir = dirname(fileURLToPath(import.meta.url))
const attendanceFilePath = join(dir, '../../..', 'attendance.toml')

export type Attendance = {
	weeks: Week[]
}

export type Week = {
	name: string
	studies: Study[]
}

export type Study = {
	weekday: string
	date: string // YYYY-MM-DD
	attended: string[] // user IDs
}

export enum MarkResult {
	AlreadyMarked,
	Marked,
	NotAMember,
}

/**
 * `attendance.toml` file looks like this:
 *
 * [[weeks]]
 * name = "Week 1"
 *
 * [[weeks.studies]]
 * weekday = "saturday"
 * date = 2025-11-08
 * attended = [ ... <array of attended user ids> ... ]
 *
 * [[weeks.studies]]
 * weekday = "sunday"
 * date = 2025-11-09
 * attended = [ ... <array of attended user ids> ... ]
 *
 * [[weeks]]
 * name = "Week 2"
 *
 * # ...
 */
export class AttendanceStore {
	#tomlData: Attendance = { weeks: [] }
	#studyData: StudyData

	constructor(studyData: StudyData) {
		this.#studyData = studyData
	}

	async load() {
		const raw = await readFile(attendanceFilePath, 'utf-8')

		this.#tomlData = parse(raw) as unknown as Attendance
	}

	async save() {
		const tomlString = stringify(this.#tomlData)

		await writeFile(attendanceFilePath, tomlString, 'utf-8')
	}

	// retrieve and return this week's entry from internal toml data.
	// create a new one if not exists.
	thisWeekEntry(): Week {
		const date = new Date()

		const weekName = dateToStudyWeek(this.#studyData.start_date, date)

		let existingWeek = this.#tomlData.weeks.find(
			week => week.name === weekName
		)

		if (!existingWeek) {
			existingWeek = {
				name: weekName,
				studies: [],
			}

			this.#tomlData.weeks.push(existingWeek)
		}

		return existingWeek
	}

	// retrieve and return today's study entry from internal toml data.
	// create a new one if not exists.
	todayEntry(): Study {
		const dateKey = dateToDatekey()

		const week = this.thisWeekEntry()

		let existingStudy = week.studies.find(study => study.date === dateKey)

		if (!existingStudy) {
			// create new study entry for today
			const weekday = datekeyWeekday(dateKey)

			existingStudy = {
				weekday,
				date: dateKey,
				attended: [],
			}

			week.studies.push(existingStudy)
		}

		return existingStudy
	}

	async mark(userId: string): Promise<MarkResult> {
		await this.load()

		const todayStudy = this.todayEntry()

		// return if not a member
		const isMember = this.#studyData.members.some(
			member => member.id === userId
		)
		if (!isMember) return MarkResult.NotAMember

		// skip if already marked
		if (todayStudy.attended.includes(userId))
			return MarkResult.AlreadyMarked

		todayStudy.attended.push(userId)

		await this.save()

		return MarkResult.Marked
	}

	// load and return today's study entry
	async getToday(): Promise<Study> {
		await this.load()
		return this.todayEntry()
	}

	// reset today's attendance
	async resetToday(): Promise<void> {
		await this.load()
		const today = this.todayEntry()
		today.attended = []
		await this.save()
	}
}
