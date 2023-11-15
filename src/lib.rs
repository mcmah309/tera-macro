extern crate proc_macro;

use proc_macro2::TokenStream;
use serde_json::Value;
use syn::LitStr;
use syn::parse::{Parse, ParseStream};
use tera::{Context, Tera};
use unicode_segmentation::UnicodeSegmentation;

#[proc_macro]
pub fn tera(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens_string = tokens.to_string();
    if tokens_string.is_empty() {
        return tokens;
    }
    let (context_string, template_string) = parse_into_context_and_code(&tokens_string, tokens);
    println!("context: {}", context_string);
    println!("template: {}", template_string);
    let value: Value = serde_json::from_str(&context_string).expect("Could not parse context in valid json.");
    println!("value: {}", value);
    let mut tera = Tera::default();
    let refined_template = remove_space_between_tera_brackets_added_by_parsing(&template_string);
    println!("refined_template: {}", refined_template);
    tera.add_raw_template("tera", &refined_template).expect("The template was not valid.");
    let context = Context::from_value(value).expect("Tera failed to create Context from json value.");
    let output: String = tera.render("tera", &context).expect("Could not render the template");
    println!("output: {}", output);
    let token_stream: TokenStream = syn::parse_str(&output)
        .expect("Could not converted the rendered output into a valid token stream");
    proc_macro::TokenStream::from(token_stream)
}


fn remove_space_between_tera_brackets_added_by_parsing(input: &str) -> String {
    let graphemes = input.graphemes(true);
    let mut graphemes_it = graphemes.into_iter();
    let Some(mut prev) = graphemes_it.next() else {
        return String::new();
    };
    let Some(mut this) = graphemes_it.next() else {
        return prev.into();
    };
    let mut result: Vec<&str> = Vec::with_capacity(input.len());
    result.push(prev);

    while let Some(next) = graphemes_it.next() {
        match (prev, this, next) {
            ("{", " ", "{") => (),
            ("{", " ", "%") => (),
            ("{", " ", "#") => (),
            ("}", " ", "}") => (),
            ("%", " ", "}") => (),
            ("#", " ", "}") => (),
            _ => result.push(this)
        }
        prev = this;
        this = next;
    }
    result.push(this);
    result.join("")
}

//************************************************************************//

/// parses into the context and rust code strings. Context is either originally json or string json
fn parse_into_context_and_code(input_as_string: &str, input_as_tokens: proc_macro::TokenStream) -> (String, String) {
    let first = input_as_string.trim().graphemes(true).next().expect("input is empty");
    return match first {
        "{" => parse_json_context(input_as_string),
        _ => {
            let (left, right) =
                parse_string_context(input_as_tokens);
            return (left.to_owned(), right.to_owned());
        }
    };
}


struct TeraMacroStringInput {
    context: LitStr,
    rust_code: TokenStream,
}

impl Parse for TeraMacroStringInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let context: LitStr = input.parse().expect("Tried to treat context as a String, but was not a String");
        input.parse::<syn::token::Comma>().expect("Context and code are not separated by a comma");
        let rust_code = input.parse()?;

        Ok(TeraMacroStringInput { context, rust_code })
    }
}

fn parse_string_context(tokens: proc_macro::TokenStream) -> (String, String) {
    let parsed: TeraMacroStringInput = syn::parse(tokens).expect("Could not parse context from code.");
    (parsed.context.value(), parsed.rust_code.to_string())
}

fn parse_json_context(input: &str) -> (String, String) {
    let mut brace_count = 0;
    let mut index = 0;
    let graphemes = input.graphemes(true).collect::<Vec<&str>>();

    for grapheme in graphemes.as_slice() {
        if *grapheme == "{" {
            brace_count += 1;
        } else if *grapheme == "}" {
            brace_count -= 1;
        }
        index += 1;
        if brace_count <= 0 {
            break;
        }
    }

    if brace_count != 0 {
        panic!("Unbalanced braces in context");
    }
    if index == 0 {
        panic!("No context was found");
    }

    let (left, rust_code_with_comma) = graphemes.split_at(index);
    let context = left.join("");
    let mut index = 0;
    for grapheme in rust_code_with_comma.iter() {
        if grapheme.chars().all(char::is_whitespace) {
            continue;
        }
        if *grapheme == "," {
            break;
        } else {
            panic!("Context and code are not separated by a comma.")
        }
    }
    index += 1;
    if index >= rust_code_with_comma.len() {
        panic!("No Rust code was provided.")
    }

    (context, rust_code_with_comma[index..].join(""))
}