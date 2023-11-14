use tera_macro::tera;

tera! {
    r#"{ "x" : "1" }"#,
    const w: usize = {{ x }};
}


#[test]
fn it_works() {
    tera! {
        r#"{ "x" : "2" }"#,
        let y: usize = {{ x }};
    }
    assert_eq!(y, 2);
    assert_eq!(w, 1);
}