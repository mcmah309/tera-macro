#![feature(fmt_internals)]
extern crate proc_macro;

use std::ops::Add;

use anyhow::{bail, Context};
use proc_macro2::{Span, TokenStream};
use serde_json::{Map, Number, Value};
use syn::{Lit, LitStr};
use syn::__private::ToTokens;
use syn::parse::{Parse, ParseStream};
use tera::Tera;
use unicode_segmentation::UnicodeSegmentation;

#[proc_macro]
pub fn tera(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (context, template) = parse_into_context_and_template(tokens)
        .expect("Failed to parse context and template from macro.");
    let mut tera = Tera::default();
    let refined_template = remove_space_between_tera_brackets_added_by_parsing(&template);
    if cfg!(feature = "debug_print") {
        println!("refined_template: {}", refined_template);
    }
    tera.add_raw_template("tera", &refined_template).expect("The template was not valid template to add to Tera.");
    let context = tera::Context::from_value(context).expect("Tera failed to create Context from json value.");
    let render_output: String = tera.render("tera", &context).expect("Could not render the template with context");
    if cfg!(feature = "debug_print") {
        println!("render_output: {}", render_output);
    }
    let token_stream: TokenStream = syn::parse_str(&render_output)
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
fn parse_into_context_and_template(input_as_tokens: proc_macro::TokenStream) -> anyhow::Result<
    (Value,
     String)> {
    let parsed = get_parse_macro_input(input_as_tokens)?;
    if cfg!(feature = "debug_print") {
        println!("Starting to get context and template");
    }
    let context: Value = match parsed.context {
        FirstArg::Json(bracket) => {
            let mut json: Map<String, Value> = Map::new();
            for (i, elem) in bracket.elems.iter().enumerate() {
                json.insert("val".to_owned().add(&*i.to_string()), Value::String(elem.to_token_stream().to_string()));
            }
            Value::Object(json)
        }
        FirstArg::ArrayOfVals(brace) => to_value(brace)?,
        FirstArg::String(string) => to_value(string)?,
        FirstArg::Val(val) => {
            let mut json: Map<String, Value> = Map::new();
            json.insert("val".to_owned(), val);
            Value::Object(json)
        }
    };
    if cfg!(feature = "debug_print") {
        println!("Successfully go context and template");
        println!("context: {}", context);
        println!("template: {}", parsed.template);
    }
    Ok((context, parsed.template))

    // Err("Could not determine the type of the context.".into())
}

fn to_value(to_json: String) -> anyhow::Result<Value> {
    if cfg!(feature = "debug_print") {
        println!("String to be converted to json: {}", to_json);
    }
    let json_result: anyhow::Result<Value> = serde_json::from_str(&to_json).context("String to json failed");
    json_result
}

enum FirstArg {
    // Brace
    Json(syn::ExprArray),
    // string before jsonify
    String(String),
    // Bracket
    ArrayOfVals(String),
    // A literal or an Ident
    Val(Value),
}

struct TeraMacroInput {
    context: FirstArg,
    template: String,
}

enum LitOrIdent {
    Lit(Lit),
    Ident(syn::Ident),
}

fn to_syn_error<E>(e: E) -> syn::Error
    where
        E: std::fmt::Display,
{
    syn::Error::new(Span::call_site(), format!("{}", e))
}

impl Parse for TeraMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let context: FirstArg;
        let template: String;
        if lookahead.peek(syn::token::Bracket) {
            if cfg!(feature = "debug_print") {
                println!("First argument has been identified as Json.");
            }
            context = input.parse().map(FirstArg::Json)?;
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
        } else if lookahead.peek(LitStr) {
            if cfg!(feature = "debug_print") {
                println!("First argument has been identified as Json string.");
            }
            context = FirstArg::String(input.parse::<syn::LitStr>()?.value());
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
        } else if lookahead.peek(syn::token::Brace) {
            if cfg!(feature = "debug_print") {
                println!("First argument has been identified as a Json array of vals.");
            }
            let json_parse_result = parse_json_context(input.parse::<TokenStream>()?.to_string())
                .map_err(to_syn_error)?;
            let (context_string, template_string) = json_parse_result;
            context = FirstArg::ArrayOfVals(context_string);
            template = template_string;
        } else {
            let first_arg = if lookahead.peek(Lit) {
                if cfg!(feature = "debug_print") {
                    println!("First argument has been identified as a single Lit val.");
                }
                LitOrIdent::Lit(input.parse()?)
            } else if lookahead.peek(syn::Ident) {
                if cfg!(feature = "debug_print") {
                    println!("First argument has been identified as a single Ident val.");
                }
                LitOrIdent::Ident(input.parse()?)
            } else {
                let mut err = lookahead.error();
                err.combine(syn::Error::new(Span::call_site(), "Context was expected to be a \
                  literal or ident but was neither.".to_owned()));
                return Err(err);
            };
            let val: Value = match first_arg {
                LitOrIdent::Ident(ident) => {
                    Value::String(ident.to_string())
                },
                LitOrIdent::Lit(lit) => {
                    match lit {
                        Lit::Str(string_lit) => Value::String(string_lit.value()),
                        Lit::ByteStr(byte_str_lit) => Value::Array(
                            byte_str_lit
                                .value()
                                .iter()
                                .map(|&byte| Value::Number(Number::from(byte)))
                                .collect(),
                        ),
                        Lit::Byte(byte_lit) => Value::Number(Number::from(byte_lit.value())),
                        Lit::Char(char_lit) => Value::String(char_lit.value().to_string()),
                        Lit::Int(int_lit) => Value::Number(
                            Number::from(int_lit.base10_parse::<i64>().map_err(to_syn_error)?),
                        ),
                        Lit::Float(float_lit) => Value::Number(
                            float_lit
                                .base10_parse::<f64>()
                                .map(Number::from_f64)
                                .map_err(to_syn_error)?
                                .ok_or(to_syn_error("Not float 64 compatible"))?,
                        ),
                        Lit::Bool(bool_lit) => Value::Bool(bool_lit.value),
                        Lit::Verbatim(verbatim_lit) => Value::String(verbatim_lit.to_string()),
                        _ => Err(to_syn_error("Lit conversion not defined"))?,
                    }
                }
            };
            context = FirstArg::Val(val);
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
        }
        Ok(TeraMacroInput { context, template })
    }
}


fn get_parse_macro_input(tokens: proc_macro::TokenStream) -> anyhow::Result<TeraMacroInput> {
    syn::parse(tokens).context("Could not parse context from macro.")
}

fn parse_json_context(input: String) -> anyhow::Result<(String, String)> {
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
        bail!("Unbalanced braces in json context");
    }
    if index == 0 {
        bail!("No context was found when parsing json.");
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
            bail!("Json context and code are not separated by a comma.");
        }
    }
    index += 1;
    if index >= rust_code_with_comma.len() {
        bail!("No Rust code was provided with Json context.");
    }

    Ok((context, rust_code_with_comma[index..].join("")))
}