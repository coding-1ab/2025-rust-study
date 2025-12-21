import { AttendanceStore } from './attendance-store'
import { Bot } from './bot'
import { readStudyData } from './study'

const studyData = await readStudyData()
const store = new AttendanceStore(studyData)

const bot = new Bot(studyData, store)

bot.login()
