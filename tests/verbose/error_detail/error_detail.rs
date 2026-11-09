use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The display shows what made an error under its line when contents are allowed, and says it is
/// hidden otherwise; the line of the error never holds it.
#[tokio::test]
async fn error_detail() {
    let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
    for allowed in [true, false] {
        let (mut p, _) = program(here(), "error_detail", crate::common::run_judge::mock(verdict.clone())).await;
        let screen = show(&mut p, VerboseLevel::Steps, allowed, false);
        assert!(p.run("hi").await.is_err());
        let lines = screen.lines();
        let at = lines
            .iter()
            .position(|l| l.contains("✗ bad_value"))
            .unwrap_or_else(|| panic!("{lines:?}"));
        assert!(!lines[at].contains("diabetes"), "{lines:?}");
        let under = lines.get(at + 1).cloned().unwrap_or_default();
        if allowed {
            assert!(under.contains("detail:") && under.contains("diabetes"), "{lines:?}");
        } else {
            assert!(under.contains("contents hidden"), "{lines:?}");
            assert!(!screen.text().contains("diabetes"), "{lines:?}");
        }
    }
}
