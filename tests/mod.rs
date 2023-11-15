use std::fmt::Debug;

use tera_macro::tera;

// #[test]
// fn it_works() {
//     tera! {
//         { "x" : "1" },
//         let result: usize = {{ x }};
//     }
//     assert_eq!(result, 1);
//
//     tera! {
//         r#"{ "x" : "2" }"#,
//         let result: usize = {{ x }};
//     }
//     assert_eq!(result, 2);
// }
//
// #[test]
// fn for_loop() {
//     tera! {
//         { "x" : 20 },
//         let result: Vec<usize> = vec![
//         {% for number in range(end=x) %}
//             {% if number % 2 == 0 %}
//                 {{ number }},
//             {% endif %}
//         {% endfor %}
//         ];
//     }
//
//     let expected: Vec<usize> = vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18];
//     assert_eq!(result, expected);
// }
//
// macro_rules! inside_another_macro {
//     ($($rest:tt)*) => {
//         tera!($($rest)*);
//         //$($rest)*
//     };
// }
//
// inside_another_macro! {
//         r#"{ "value" : "313" }"#,
// struct TestStruct {
//     val: i64
// }
//
// impl Default for TestStruct {
//     fn default() -> Self {
//         TestStruct{ val : {{ value }} }
//     }
// }
// }
//
// #[test]
// fn inside_another_macro() {
//     let result = TestStruct::default();
//     assert_eq!(result.val, 313);
// }


// macro_rules! tera2 {
//     ($val:ident) => {tera!{
//         {"name" : "$val"},
//
//         {% macro pascal_to_camel(s) %}
//         {% set_global pascal =  "" %}
//         {% for c in s %}
//             {% if loop.first %}
//                 {% set_global pascal = pascal ~ c | lower %}
//             {% else %}
//                 {% set_global pascal = pascal ~ c %}
//             {% endif %}
//         {% endfor %}
//         {{ pascal }}
//     {% endmacro pascal_to_camel %}
//     struct {{ self::pascal_to_camel(s=name) }};
//     }}
// }
//
// tera2! { ToPascalCase2 }

tera! {
    {"name" : "ToPascalCase2"},

    {% macro pascal_to_camel(s) %}
        {% set_global pascal =  "" %}
        {% for c in s %}
            {% if loop.first %}
                {% set_global pascal = pascal ~ c | lower %}
            {% else %}
                {% set_global pascal = pascal ~ c %}
            {% endif %}
        {% endfor %}
        {{ pascal }}
    {% endmacro pascal_to_camel %}
    struct {{ self::pascal_to_camel(s=name) }};
}


#[test]
fn to_pascal_case_macro() {
    let result = toPascalCase;
}

// #[test]
// fn to_pascal_case_macro2() {
//     let result = toPascalCase2;
// }