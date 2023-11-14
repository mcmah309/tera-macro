extern crate proc_macro;

use proc_macro2::TokenStream;
use serde_json::Value;
use syn::LitStr;
use syn::parse::{Parse, ParseStream, Result};
use tera::{Context, Tera};
use unicode_segmentation::UnicodeSegmentation;

struct TeraMacroInput {
    context: LitStr,
    rust_code: TokenStream,
}

impl Parse for TeraMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let json: LitStr = input.parse()?;
        input.parse::<syn::token::Comma>()?;
        let rust_code = input.parse()?;

        Ok(TeraMacroInput { context: json, rust_code })
    }
}

#[proc_macro]
pub fn tera(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as TeraMacroInput);
    let template = remove_space_added_by_parsing(&input.rust_code.to_string());
    println!("template: {:?}", template);
    let mut tera = Tera::default();
    tera.add_raw_template("tera", &template).expect("The template was not valid.");
    let json_string: String = input.context.value();
    let value: Value = serde_json::from_str(&json_string).expect("The Context was not valid json.");
    println!("value: {:?}", value);
    let context = Context::from_value(value).expect("Tera failed to create Context from json value.");
    let output: String = tera.render("tera", &context).expect("Could not render the template");
    println!("output: {:?}", output);
    let token_stream: TokenStream = syn::parse_str(&output).expect("Could not converted the rendered output into a \
    valid token stream");
    proc_macro::TokenStream::from(token_stream)
}


fn remove_space_added_by_parsing(input: &str) -> String {
    let graphemes = UnicodeSegmentation::graphemes(input, true);
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
