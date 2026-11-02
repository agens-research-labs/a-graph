use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An approved message goes on unchanged.
#[tokio::test]
async fn approved_passes() {
    let (p, seen) = program(here(), "approved_passes", approved()).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(order(&seen), "SJZE");
    // the judge read the message of s with its prompt; z received the same message
    assert_eq!(asked(&seen, 'J'), "J(S(hi))");
    assert_eq!(asked(&seen, 'Z'), "Z(S(hi))");
    // the judge produces nothing for the final output
    assert_eq!(
        items(&out),
        [pair("s", "S(hi)"), pair("z", "Z(S(hi))"), pair("e", "E(Z(S(hi)))")]
    );
}
