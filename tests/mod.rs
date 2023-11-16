use tera_macro::tera;

#[test]
fn it_works() {
    tera! {
        { "x" : "1" },
        let result: usize = {{ x }};
    }
    assert_eq!(result, 1);

    tera! {
        [todd, 2],
        let {{ val0 }}: usize = {{ val1 }};
    }
    assert_eq!(todd, 2);

    tera! {
        bob,
        let {{ val }}: usize = 1;
    }
    assert_eq!(bob, 1);

    tera! {
        "dave",
        let {{ val }}: usize = 1;
    }
    assert_eq!(dave, 1);

    tera! {
        "john",
        let john = "this is john";
    }
    assert_eq!(john, "this is john");
}

#[test]
fn for_loop() {
    tera! {
        20,
        let result: [usize; 10] = [
        {% for number in range(end=val) %}
            {% if number % 2 == 0 %}
                {{ number }},
            {% endif %}
        {% endfor %}
        ];
    }

    let expected: [usize; 10] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18];
    assert_eq!(result, expected);
}

macro_rules! inside_another_macro {
    ($($rest:tt)*) => {
        tera!($($rest)*);
        //$($rest)*
    };
}

inside_another_macro! {
        { "value" : "313" },
struct TestStruct {
    val: i64
}

impl Default for TestStruct {
    fn default() -> Self {
        TestStruct{ val : {{ value }} }
    }
}
}

#[test]
fn inside_another_macro() {
    let result = TestStruct::default();
    assert_eq!(result.val, 313);
}


tera! {
    ToPascalCase,

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
    struct {{ self::pascal_to_camel(s=val) }};
}


#[test]
fn to_pascal_case_single_macro() {
    let _ = toPascalCase;
}


macro_rules! to_pascal_case {
    ($val:ident) => {tera!{
        $val,

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
    struct {{ self::pascal_to_camel(s=val) }};
    }}
}

to_pascal_case! { ToPascalCase2 }

#[test]
fn to_pascal_case_inside_another_macro() {
    let _ = toPascalCase2;
}