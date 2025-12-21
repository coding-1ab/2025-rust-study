import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { parse } from 'smol-toml'

export type Identifiable = { id: string }

export type Member = Identifiable & {
	nickname: string
	preferred_study_weekday: string
}

export type StudyData = {
	name: string
	start_date: Date
	end_date: Date

	study_time: Array<{
		weekday: string
		start_time: string
		end_time: string
	}>

	lecturer: Identifiable
	members: Array<Member>
}

export const readStudyData = async (): Promise<StudyData> => {
	const studyDataPath = resolve(import.meta.dirname, '../../../study.toml')

	const studyData = (await readFile(studyDataPath, 'utf-8').then(
		parse
	)) as StudyData

	return studyData
}

export const studyData = await readStudyData()
