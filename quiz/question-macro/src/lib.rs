use proc_macro::{Span, TokenStream};
use quote::quote;
use std::fs;
use std::io::Read;

#[proc_macro]
pub fn include_questions(path: TokenStream) -> TokenStream {
    let path: syn::LitStr = syn::parse(path).expect("Only rust string is accepted");
    let span = Span::call_site();
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    let location = format!("{}/../{}", MANIFEST_DIR, path.value());

    let mut struct_initializations: Vec<proc_macro2::TokenStream> = vec![];
    for question_file in fs::read_dir(location).expect("Unable to read directory") {
        let question_file = question_file.expect("Unable to access file");
        let file_name = question_file.file_name();
        let Some(file_name) = file_name.to_str() else {
            eprintln!("Invalid file name detected: {:?}", file_name);
            continue;
        };
        if !file_name.ends_with(".md") {
            continue;
        }

        let contents = {
            let mut file = fs::File::open(question_file.path()).expect("Unable to open file");
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .expect("Unable to read file");
            buffer
        };

        let mut title = None;
        let mut parsing_mode = ParseMode::Title;
        let mut description = String::new();
        let mut code = String::new();
        let mut choices = vec![];
        let mut answer = None;
        for line in contents.lines() {
            if line.is_empty() {
                match parsing_mode {
                    ParseMode::Title | ParseMode::Description | ParseMode::Choices => continue,
                    _ => {}
                }
            } else if line.starts_with("```rs") {
                parsing_mode = ParseMode::Code;
                continue;
            } else if line == "```" {
                parsing_mode = ParseMode::Choices;
                continue;
            } else if line.starts_with("- [") && matches!(parsing_mode, ParseMode::Description) {
                parsing_mode = ParseMode::Choices;
            }

            match parsing_mode {
                ParseMode::Title => {
                    if line.starts_with("# ") {
                        title = Some(line[2..].to_string());
                        parsing_mode = ParseMode::Description;
                    } else {
                        panic!("File {} has invalid title!:\n{}", file_name, line);
                    }
                }
                ParseMode::Description => {
                    description.push_str(line);
                    description.push('\n');
                }
                ParseMode::Code => {
                    code.push_str(line);
                    code.push('\n');
                }
                ParseMode::Choices => {
                    let pattern1 = "- [ ] ";
                    let pattern2 = "- [x] ";
                    assert_eq!(pattern1.len(), pattern2.len());
                    if let None = line.find(pattern1).or_else(|| {
                        if answer.is_some() {
                            panic!("File {} contains duplicate x markers!", file_name);
                        }
                        answer = Some(choices.len());
                        line.find(pattern2)
                    }) {
                        panic!("File {} contains invalid line: {}", file_name, line);
                    }

                    let bracket_end = line.rfind(']').unwrap();
                    let choice = if bracket_end == line.len() - 1 {
                        let bracket_start_pattern = ": [";
                        let bracket_start = line
                            .rfind(bracket_start_pattern)
                            .expect(format!("File {} contains line: {}", file_name, line).as_str());
                        let choice_label = &line[pattern1.len()..bracket_start];
                        let text_start = bracket_start + bracket_start_pattern.len();
                        let text_contents = &line[text_start..bracket_end];
                        ChoiceKind::Subjective(choice_label.to_string(), text_contents.to_string())
                    } else {
                        ChoiceKind::Choice(line[pattern1.len()..].to_string())
                    };
                    choices.push(choice);
                }
            }
        }

        let Some(title) = title else {
            panic!("File {} does not contain title!", file_name);
        };
        let Some(answer) = answer else {
            panic!("File {} does not contain answer with x mark!", file_name);
        };

        let choices: Vec<proc_macro2::TokenStream> = choices
            .into_iter()
            .map(|choice| match choice {
                ChoiceKind::Choice(label) => {
                    quote! {
                        Answer::Choice { label: #label }
                    }
                }
                ChoiceKind::Subjective(label, text) => {
                    let literal = syn::LitStr::new(&text, span.into());
                    quote! {
                        Answer::Subjective { label: #label, value: #literal }
                    }
                }
            })
            .collect();
        let initializer = quote! {
            Question {
                name: #title,
                description: #description,
                code: #code,
                choices: &[
                    #(#choices),*
                ],
                answer: #answer
            }
        };

        struct_initializations.push(initializer);
    }

    let output = quote! {
        &[#(#struct_initializations),*]
    }
    .into();
    output
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ParseMode {
    Title,
    Description,
    Code,
    Choices,
}

enum ChoiceKind {
    Choice(String),       // 객관식
    Subjective(String, String), // 주관식
}
