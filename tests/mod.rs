use tera_macro::tera;

#[test]
fn it_works() {
    tera! {
        { "x" : "1" },
        let result: usize = {{ x }};
    }
    assert_eq!(result, 1);

    tera! {
        r#"{ "x" : "2" }"#,
        let result: usize = {{ x }};
    }
    assert_eq!(result, 2);
}

#[test]
fn for_loop() {
    tera! {
        { "x" : 20 },
        let result: Vec<usize> = vec![
        {% for number in range(end=x) %}
            {% if number % 2 == 0 %}
                {{ number }},
            {% endif %}
        {% endfor %}
        ];
    }

    let expected: Vec<usize> = vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18];
    assert_eq!(result, expected);
}