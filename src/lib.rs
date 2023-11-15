#![feature(fmt_internals)]
extern crate proc_macro;

use std::fmt::Display;
use std::ops::Add;

use proc_macro2::{Span, TokenStream};
use serde_json::{Map, Number, Value};
use syn::{Lit, LitStr};
use syn::__private::ToTokens;
use syn::parse::{Parse, ParseStream};
use tera::{Context, Tera};
use unicode_segmentation::UnicodeSegmentation;

#[proc_macro]
pub fn tera(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens_string = tokens.to_string();
    if tokens_string.is_empty() {
        return tokens;
    }
    let (context, template) = parse_into_context_and_template(&tokens_string, tokens).expect("TODO: panic message");
    println!("context: {}", context);
    println!("template: {}", template);
    let mut tera = Tera::default();
    let refined_template = remove_space_between_tera_brackets_added_by_parsing(&template);
    println!("refined_template: {}", refined_template);
    tera.add_raw_template("tera", &refined_template).expect("The template was not valid.");
    let context = Context::from_value(context).expect("Tera failed to create Context from json value.");
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
fn parse_into_context_and_template(input_as_string: &str, input_as_tokens: proc_macro::TokenStream) -> syn::Result<
    (Value,
     String)> {
    let parsed = get_parse_macro_input(input_as_tokens)?;
    println!("we back");
    let value: Value = match parsed.context {
        FirstArg::Bracket(bracket) => {
            let mut json: Map<String, Value> = Map::new();
            for (i, elem) in bracket.elems.iter().enumerate() {
                json.insert("val".to_owned().add(&*i.to_string()), Value::String(elem.to_token_stream().to_string()));
            }
            Value::Object(json)
        }
        FirstArg::Brace(brace) => to_value(brace)?,
        FirstArg::String(string) => to_value(string)?,
        FirstArg::Val(val) => {
            let mut json: Map<String, Value> = Map::new();
            json.insert("val".to_owned(), val);
            Value::Object(json)
        }
    };
    println!("it ok?");
    Ok((value, parsed.template))

    // Err("Could not determine the type of the context.".into())
}

fn to_value(part: String) -> syn::Result<Value> {
    println!("token_string: {}", part);
    let json_result: syn::Result<Value> = serde_json::from_str(&part).map_err(|err| {
        let mut err = syn::Error::new(Span::call_site(), err.to_string());
        err.add_message("String to json failed".to_owned());
        err
    });
    json_result
}

enum FirstArg {
    Bracket(syn::ExprArray),
    // string before jsonify
    String(String),
    // string before jsonify
    Brace(String),
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

fn handle_parse_error<E>(e: E) -> syn::Error
    where
        E: std::fmt::Display,
{
    syn::Error::new(Span::call_site(), format!("Could not map literal to json value type: {}", e))
}

impl Parse for TeraMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        println!("parsing it up");
        let lookahead = input.lookahead1();
        let context: FirstArg;
        let template: String;
        println!("n");
        if lookahead.peek(syn::token::Bracket) {
            println!("no");
            context = input.parse().map(FirstArg::Bracket)?;
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
        } else if lookahead.peek(LitStr) {
            println!("not");
            context = FirstArg::String(input.parse::<syn::LitStr>()?.value());
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
        } else if lookahead.peek(syn::token::Brace) {
            println!("not f");
            let (context_string, template_result) = parse_json_context(input.parse::<TokenStream>()?.to_string())?;
            println!("context_before: {}", context_string);
            context = FirstArg::Brace(context_string);
            template = template_result;
        } else {
            let first_arg = if lookahead.peek(Lit) {
                LitOrIdent::Lit(input.parse()?)
            } else if lookahead.peek(syn::Ident) {
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
                            Number::from(int_lit.base10_parse::<i64>().map_err(handle_parse_error)?),
                        ),
                        Lit::Float(float_lit) => Value::Number(
                            float_lit
                                .base10_parse::<f64>()
                                .map(Number::from_f64)
                                .map_err(handle_parse_error)?
                                .ok_or(handle_parse_error("Not float 64 compatible"))?,
                        ),
                        Lit::Bool(bool_lit) => Value::Bool(bool_lit.value),
                        Lit::Verbatim(verbatim_lit) => Value::String(verbatim_lit.to_string()),
                        _ => Err(handle_parse_error("Lit conversion not defined"))?,
                    }
                }
            };
            context = FirstArg::Val(val);
            input.parse::<syn::token::Comma>()?;
            template = input.parse::<TokenStream>()?.to_string();
            // let mut val_tokens = TokenStream::new();
            // while !input.peek(syn::token::Comma) {
            //     let token: proc_macro2::TokenTree = input.parse()?;
            //     val_tokens.extend(Some(token));
            // }
            // input.parse::<syn::token::Comma>()?;
            // println!("last");
            // context = FirstArg::Val(val_tokens.to_string());
            // template = input.parse::<TokenStream>()?.to_string();

            // let string: String = input.parse::<TokenStream>()?.to_string();
            // let result = string.split_once(',')
            //     .ok_or(syn::Error::new(Span::call_site(), "Context could not be found, make sure there \
            //     is a comma between the context and template code.".to_owned()))
            //     .map(|(context, template)| (FirstArg::Val(context.to_owned()), template.to_owned()))?;
            // context = result.0;
            // template = result.1;
        }
        Ok(TeraMacroInput { context, template })
    }
}


fn get_parse_macro_input(tokens: proc_macro::TokenStream) -> syn::Result<TeraMacroInput> {
    syn::parse(tokens)
        .map_err(|mut err| {
            err.add_message("Could not parse context from macro.".to_owned());
            return err;
        })
}

fn parse_json_context(input: String) -> syn::Result<(String, String)> {
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
        return Err(syn::Error::new(Span::call_site(), "Unbalanced braces in json context".to_owned()));
    }
    if index == 0 {
        return Err(syn::Error::new(Span::call_site(), "No context was found when parsing json".to_owned()));
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
            return Err(syn::Error::new(Span::call_site(), "Json context and code are not separated by a comma.".to_owned()));
        }
    }
    index += 1;
    if index >= rust_code_with_comma.len() {
        return Err(syn::Error::new(Span::call_site(), "No Rust code was provided with Json context.".to_owned()));
    }

    Ok((context, rust_code_with_comma[index..].join("")))
}


trait AddMessageToSynError {
    fn add_message(&mut self, message: String);
}

impl AddMessageToSynError for syn::Error {
    fn add_message(&mut self, message: String) {
        self.combine(syn::Error::new(Span::call_site(), message))
    }
}