#![warn(clippy::all)]

use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use chrono::Local;
use question_macro::include_questions;
use rand::TryRngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tracing::error;
use url::form_urlencoded::byte_serialize;


pub const FIVE_MINUTES: Duration = Duration::from_mins(5);
static QUIZ_TEMPLATE: &str = include_str!("../question_template.html");
static FINISH_TEMPLATE: &str = include_str!("../finish_template.html");
pub static QUESTIONS: &[Question] = include_questions!("questions");

pub static ICON: &[u8] = include_bytes!("../favicon.png");
pub static MIRACODE: &[u8] = include_bytes!("../Miracode.ttf");
pub static PRETENDARD_VARIABLE: &[u8] = include_bytes!("../PretendardVariable.woff2");

pub fn render_question(question: &Question, index: usize) -> Html<String> {
    static CHOICE_OPTION_TEMPLATE: &'static str = r#"
<input type="radio" id="optionCHOICE_INDEX" name="option" value="CHOICE_INDEX" style="cursor: pointer" onchange="updateAnswer()">
<label for="optionCHOICE_INDEX" style="cursor: pointer">CHOICE_LABEL</label>
<br/>
"#;

    static TEXT_OPTION_TEMPLATE: &'static str = r#"
<input type="radio" id="optionCHOICE_INDEX" name="option" value="CHOICE_INDEX" style="cursor: pointer" onchange="updateAnswer()">
<label for="optionCHOICE_INDEX" style="cursor: pointer">CHOICE_LABEL:</label>
<input type="text" id="optionCHOICE_INDEXtext" style="font-family: 'Pretendard Variable',serif; font-weight: 200; background-color: #121212; border-radius: 4px; border-color: transparent; color: white; width: 128px" onchange="updateAnswer()">
<br/>
"#;

    static CODE_TEMPLATE: &'static str = r#"
<pre><code
    id="code"
    class="rust"
    style="background-color: #121212;
    border-radius: 15px;
    text-align: left;
    margin: 10px;
    padding-left: 40px;
    padding-top: 24px;
    font-family: Miracode,monospace"
>QUESTION_CODE_CONTENTS</code></pre>
"#;

    let mut choices = String::new();
    for (i, choice) in question.choices.iter().enumerate() {
        let (template, label) = match choice {
            Answer::Choice { label } => (CHOICE_OPTION_TEMPLATE, label),
            Answer::Subjective { label, .. } => (TEXT_OPTION_TEMPLATE, label),
        };

        choices.push_str(
            &template
                .replace("CHOICE_INDEX", format!("{}", i).as_str())
                .replace("CHOICE_LABEL", label),
        );
    }
    let answer = match question.choices[question.answer] {
        Answer::Choice { .. } => question.answer.to_string(),
        Answer::Subjective { value, .. } => format!("{} {}", question.answer, value),
    };
    let description = question.description.replace('\n', "<br/>");
    let code = if !question.code.is_empty() {
        CODE_TEMPLATE.replace("QUESTION_CODE_CONTENTS", question.code)
    } else {
        String::new()
    };

    Html::from(
        QUIZ_TEMPLATE
            .replace("QUESTION_NAME", question.name)
            .replace("QUESTION_DESCRIPTION", &description)
            .replace("QUESTION_CODE", &code)
            .replace("QUESTION_CHOICES", &choices)
            .replace("QUESTION_NUMBER", &format!("{}", index + 1))
            .replace("QUESTION_COUNT", &format!("{}", QUESTIONS.len()))
            .replace("QUESTION_ANSWER", &answer),
    )
}

pub fn render_redirect() -> Html<String> {
    static REDIRECT_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="kr" xmlns="http://www.w3.org/1999/html">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">

    <title>코딩랩 Rust 스터디 중간평가</title>

    <meta property="og:site_name" content="코딩랩 Rust 스터디 중간평가"/>
    <meta property="og:title" content="코딩랩 Rust 스터디 중간평가"/>
    <link rel="icon" href="./favicon.png"/>
</head>
<body>
<script>
/* Randomize array in-place using Durstenfeld shuffle algorithm */
function shuffleArray(array) {
    for (var i = array.length - 1; i > 0; i--) {
        var j = Math.floor(Math.random() * (i + 1));
        var temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}

let newSequence = Array(RANGE_SIZE).fill().map((element, index) => index)
shuffleArray(newSequence)
const defaultSubmitted = Array(RANGE_SIZE).fill("")
const defaultCorrect = Array(RANGE_SIZE).fill(false)
document.cookie = "testSequence=" + encodeURIComponent(JSON.stringify(newSequence))
document.cookie = "testSubmitted=" + encodeURIComponent(JSON.stringify(defaultSubmitted))
document.cookie = "testCorrect=" + encodeURIComponent(JSON.stringify(defaultCorrect))
document.location.href = "./" + newSequence[0]
</script>
</body>
</html>
"#;
    let max = QUESTIONS.len();
    let contents = REDIRECT_TEMPLATE.replace("RANGE_SIZE", max.to_string().as_str());
    Html::from(contents)
}

pub fn render_finish_page(oauth_provider: Option<Arc<DiscordData>>) -> Html<String> {
    static DISCORD_UNAVAILABLE: &'static str = r#"
        <p id="no_discord" style="display: none">:p</p>
        <p style="color: yellow; font-family: 'Miracode',serif; font-weight: 600; font-size: 48px">☹</p>
        <p style="color: white; font-family: 'Pretendard Variable',serif; font-weight: 600; font-size: 48px">디스코드 API가 비활성화 되어 있습니다! 온라인 제출이 불가능합니다!</p>
        <p style="color: white; font-family: 'Pretendard Variable',serif; font-weight: 600; font-size: 24px">퀴즈 결과:</p>
"#;

    static OFFLINE_SCRIPT: &'static str = r#"
    {
        let cookieObject = getCookie()
        if (cookieObject == null) {
            document.location.href = "./"
        }
    }
    const resultList = document.getElementById("testResult")
    for (let i = 0; i < QUESTION_COUNT; i++) {
        const number = cookieObject.sequence[i]
        const result = cookieObject.correct[i]

        const text = document.createElement("a")
        text.textContent = number
        text.style.fontFamily = "Miracode"
        text.style.display = "inline-block"
        text.style.padding = "10px"
        text.href = "./" + number
        if (result) {
            text.style.color = "green"
        } else {
            text.style.color = "red"
        }
        resultList.appendChild(text)
    }
"#;

    static DISCORD_AVAILABLE: &'static str = r#"
    <p style="color: royalblue; font-family: 'Pretendard Variable',serif; font-weight: 300; font-size: 32px; cursor: pointer", onclick="submit()">제출하기</p>
"#;
    static ONLINE_SCRIPT: &'static str = r#"
    {
        let cookieObject = getCookie()
        if (cookieObject == null) {
            document.location.href = "./"
        }
    }

    function submit() {
        const cookieObject = getCookie()
        const jsonBody = JSON.stringify(cookieObject)
        fetch("./submit", {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
            },
            body: jsonBody
        }).then(response => response.text())
        .then(text => window.location.href = text)
    }
"#;

    let (submit, script) = match oauth_provider {
        None => (DISCORD_UNAVAILABLE, OFFLINE_SCRIPT),
        Some(_) => (DISCORD_AVAILABLE, ONLINE_SCRIPT),
    };

    let contents = FINISH_TEMPLATE
        .replace("LARGE_MESSAGE", "수고하셨습니다!")
        .replace("INIT_SCRIPT", script)
        .replace("QUESTION_COUNT", QUESTIONS.len().to_string().as_str())
        .replace("SUBMIT", submit);
    Html::from(contents)
}

pub async fn serve_file(file_name: &str) -> Response {
    let (content_type, bytes) = match file_name {
        "favicon.png" => ("image/png", ICON),
        "Miracode.ttf" => ("font/ttf", MIRACODE),
        "PretendardVariable.woff2" => ("font/woff2", PRETENDARD_VARIABLE),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let body = Body::from(bytes);

    Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
        .status(StatusCode::OK)
        .body(body)
        .unwrap()
}

pub async fn try_init_discord() -> Option<Arc<DiscordData>> {
    let discord_client_id = std::env::var("DISCORD_CLIENT_ID")
        .map_err(|e| {
            error!(
                "Unable to fetch environment variable DISCORD_CLIENT_ID: {:?}",
                e
            );
        })
        .ok()?;
    let discord_secret = std::env::var("DISCORD_SECRET")
        .map_err(|e| {
            error!(
                "Unable to fetch environment variable DISCORD_SECRET: {:?}",
                e
            );
        })
        .ok()?;
    let discord_redirect = std::env::var("DISCORD_REDIRECT")
        .map_err(|e| {
            error!(
                "Unable to fetch environment variable DISCORD_SECRET: {:?}",
                e
            );
        })
        .ok()?;
    let discord_redirect_encoded: String = byte_serialize(discord_redirect.as_bytes()).collect();
    let discord_guild_id = std::env::var("DISCORD_GUILD_ID")
        .map_err(|e| {
            error!(
                "Unable to fetch environment variable DISCORD_GUILD_ID: {:?}",
                e
            );
        })
        .ok()?;

    let discord_data = DiscordData {
        client_id: discord_client_id,
        secret: discord_secret,
        redirect_uri: discord_redirect,
        redirect_uri_encoded: discord_redirect_encoded,
        guild_id: discord_guild_id,
        oauth_attempts: Default::default(),
        leaderboard: Default::default(),
    };
    Some(Arc::new(discord_data))
}

pub async fn save_answer(
    discord: Arc<DiscordData>,
    sender_id: String,
    quiz_result: QuizResult,
) -> Result<(), StatusCode> {
    let json_string = serde_json::to_string_pretty(&quiz_result).unwrap();
    let sender_id_clone = sender_id.clone();

    let data_updater = async move || {
        let data = discord.as_ref();
        let mut writer = data.leaderboard.write().await;
        writer.insert(sender_id, quiz_result);
        Ok(())
    };

    let file_writer = async move || {
        let sender_id = sender_id_clone;
        let now = Local::now();
        let filename = now.format("%Y-%m-%dT%H-%M-%S.json").to_string();

        let dir_path = format!("submissions/{}", sender_id);
        let file_path = format!("submissions/{}/{}", sender_id, filename);
        if let Err(e) = create_dir_all(dir_path.as_str()).await {
            error!(
                "Unable to create submission directory at {}!\n{:?}\nData is not saved: {}",
                dir_path, e, json_string
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let mut file = match File::create(file_path.as_str()).await {
            Ok(f) => f,
            Err(e) => {
                error!(
                    "Unable to create submission file at {}!\n{:?}\nData is not saved: {}",
                    file_path, e, json_string
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        if let Err(e) = file.write_all(json_string.as_bytes()).await {
            error!(
                "Unable to write submission entry at {}!\n{:?}\nData is not saved: {}",
                file_path, e, json_string
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        if let Err(e) = file.flush().await {
            error!(
                "Unable to save submission entry at {}!\n{:?}\nData is not saved: {}",
                file_path, e, json_string
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        Ok(())
    };

    let mut join_set = JoinSet::new();
    join_set.spawn(data_updater());
    join_set.spawn(file_writer());
    let result1 = join_set.join_next().await.unwrap().unwrap();
    let result2 = join_set.join_next().await.unwrap().unwrap();

    let mut result = result1;
    if result.is_ok() {
        result = result2;
    }

    result
}

pub async fn handle_submit(discord: Arc<DiscordData>, cookie: UserCookie) -> Response {
    let mut rng = OsRng::default();
    let mut salt = [0u8; 16];
    rng.try_fill_bytes(&mut salt).unwrap();
    let salt = u128::from_le_bytes(salt);
    let salt_string = format!("{:X}", salt);
    let url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&state={}&scope=identify+guilds.members.read&prompt=none",
        discord.client_id,
        discord.redirect_uri_encoded.as_str(),
        salt_string
    );
    debug_assert_eq!(cookie.correct.len(), cookie.sequence.len());
    debug_assert_eq!(cookie.correct.len(), cookie.submitted.len());

    let score = (cookie.correct.into_iter().filter(|v| *v).count() as f32) / cookie.submitted.len() as f32;
    let answers: Result<Vec<_>, StatusCode> = cookie.sequence.into_iter().zip(cookie.submitted.into_iter()).map(|(sequence, entry)| {
        let Some(question) = QUESTIONS.get(sequence) else {
            return Err(StatusCode::BAD_REQUEST);
        };
        let name = format!("{}.md", question.name);
        let answer_value = match entry.find(' ') {
            Some(position) => {
                entry[position + 1..].to_string()
            }
            None => {
                let index: usize = entry.parse().unwrap();
                let Some(choice) = question.choices.get(index) else {
                    return Err(StatusCode::BAD_REQUEST);
                };
                match choice {
                    Answer::Choice { label } => label.to_string(),
                    Answer::Subjective { .. } => return Err(StatusCode::BAD_REQUEST),
                }
            }
        };
        Ok((name, answer_value))
    }).collect();
    let answers = match answers {
        Ok(v) => v,
        Err(e) => return e.into_response()
    };
    let quiz_result = QuizResult {
        answers,
        score,
    };

    let now = Instant::now();
    discord.oauth_attempts.write().await.insert(salt, (now, quiz_result));

    url.into_response()
}

pub async fn oauth_redirect(param: OauthRedirectUrlParams, discord: Arc<DiscordData>) -> Response {
    let Ok(salt) = u128::from_str_radix(param.state.as_str(), 16) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let mut attempts_writer = discord.oauth_attempts.write().await;
    let quiz_result = match attempts_writer.remove(&salt) {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some((time, result)) => {
            let now = Instant::now();
            let duration = now.duration_since(time);

            if duration > FIVE_MINUTES {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            result
        }
    };
    drop(attempts_writer);

    let client = reqwest::Client::new();
    let request = client
        .post("https://discord.com/api/v10/oauth2/token")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(format!(
            "grant_type=authorization_code&code={}&redirect_uri={}",
            param.code,
            discord.redirect_uri_encoded.as_str()
        ))
        .basic_auth(
            discord.client_id.as_str(),
            Some(discord.secret.as_str()),
        )
        .build()
        .unwrap();
    let response = match client.execute(request).await {
        Ok(v) => v,
        Err(e) => {
            error!("Unable to communicate with discord server!:\n{:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if response.status() != StatusCode::OK {
        error!("Discord server returned code {:?}. Response Body: {}", response.status(), response.text().await.unwrap());
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let response: OauthResponse = match response.json::<OauthResponse>().await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to decode discord server's response!:\n{:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    debug_assert!(response.scope.contains("guilds.members.read"));
    debug_assert_eq!(response.token_type, "Bearer");

    let guild_member = get_current_user_guild_profile(response.access_token.as_str(), discord.guild_id.as_str(), &client).await;
    let name = guild_member.nick.or(guild_member.user.global_name).unwrap_or(guild_member.user.username);

    let save_result = save_answer(discord, guild_member.user.id, quiz_result).await;
    match save_result {
        Ok(_) => {
            Response::builder()
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(Body::from(format!("참여해주셔서 감사합니다, {}님!", name)))
                .unwrap()
        },
        Err(code) => code.into_response()
    }
}

pub async fn get_current_user_guild_profile(
    access_token: &str,
    guild_id: &str,
    client: &reqwest::Client
) -> DiscordGuildMember {
    let url = format!("https://discord.com/api/users/@me/guilds/{}/member", guild_id);
    client.get(&url).bearer_auth(access_token).send().await.unwrap().json::<DiscordGuildMember>().await.unwrap()
}

pub struct Question {
    name: &'static str,
    description: &'static str,
    code: &'static str,
    choices: &'static [Answer],
    answer: usize,
}

pub enum Answer {
    /// 객관식
    Choice { label: &'static str },

    /// 주관식
    Subjective {
        label: &'static str,
        value: &'static str,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SubmittedAnswer {
    /// 객관식
    Choice { label: String },

    /// 주관식
    Subjective { label: String, value: String },
}

pub type SubmissionData = HashMap<String, QuizResult>;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuizResult {
    answers: Vec<(String, String)>,
    score: f32
}

#[expect(dead_code)]
pub struct DiscordData {
    client_id: String,
    secret: String,
    redirect_uri: String,
    redirect_uri_encoded: String,
    guild_id: String,
    pub oauth_attempts: RwLock<HashMap<u128, (Instant, QuizResult)>>,
    leaderboard: RwLock<SubmissionData>,
}

pub struct ServiceState {
    pub pre_rendered_questions: Vec<Html<String>>,
    pub pre_rendered_redirect: Html<String>,
    pub discord_data: Option<Arc<DiscordData>>,
    pub pre_rendered_finish_page: Html<String>,
}

#[derive(Deserialize, Debug)]
pub struct OauthRedirectUrlParams {
    code: String,
    state: String,
}

#[expect(dead_code)]
#[derive(Deserialize, Debug)]
pub struct OauthResponse {
    token_type: String,
    access_token: String,
    expires_in: usize,
    refresh_token: String,
    scope: String,
}

#[expect(dead_code)]
#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    id: String,
    username: String,
    global_name: Option<String>,
    avatar: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct DiscordGuildMember {
    user: DiscordUser,
    nick: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCookie {
    correct: Vec<bool>,
    sequence: Vec<usize>,
    submitted: Vec<String>,
}

impl Question {
    pub fn test(&self, answer: &SubmittedAnswer) -> bool {
        self.choices[self.answer].eq(answer)
    }
}

impl PartialEq<SubmittedAnswer> for Answer {
    fn eq(&self, other: &SubmittedAnswer) -> bool {
        match self {
            Answer::Choice { label } => {
                let SubmittedAnswer::Choice { label: label_other } = other else {
                    return false;
                };

                label.trim().eq_ignore_ascii_case(label_other.trim())
            }
            Answer::Subjective { label, value } => {
                let SubmittedAnswer::Subjective {
                    label: label_other,
                    value: value_other,
                } = other
                else {
                    return false;
                };

                label.trim().eq_ignore_ascii_case(label_other.trim())
                    && value.trim().eq_ignore_ascii_case(value_other.trim())
            }
        }
    }
}
