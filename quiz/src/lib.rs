use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, Response};
use question_macro::include_questions;

static TEMPLATE: &'static str = include_str!("../template.html");
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
        TEMPLATE
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
    <link rel="icon" href="/favicon.png"/>
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
document.location.href = "/" + newSequence[0]
</script>
</body>
</html>
"#;
    let max = QUESTIONS.len();
    let contents = REDIRECT_TEMPLATE.replace("RANGE_SIZE", max.to_string().as_str());
    Html::from(contents)
}

pub async fn serve_file(file_name: &str) -> Response {

    let (content_type, bytes) = match file_name {
        "favicon.png" => ("image/png", ICON),
        "Miracode.ttf" => ("font/ttf", MIRACODE),
        "PretendardVariable.woff2" => ("font/woff2", PRETENDARD_VARIABLE),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
        }
    };

    let body = Body::from(bytes);

    Response::builder()
        .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
        .status(StatusCode::OK)
        .body(body)
        .unwrap()
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
