use crate::common::cli::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media through a chain of programs writes nothing to the temporary folder: with `TMPDIR` a file, where no
/// folder can be made, the run still ends well.
#[cfg(unix)]
#[tokio::test]
async fn no_temp_dir() {
    let (model, seen) = listen(|_| Resp::ok(answer("ok"))).await;
    let yml = here().join("no_temp_dir.yml").to_string_lossy().to_string();
    let out = tokio::process::Command::new(env!("CARGO_BIN_EXE_a-graph"))
        .args([&ported(&yml, &[("model", model)]), "what is it"])
        .env("TMPDIR", here().join("files/not_a_folder"))
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap();
    assert_eq!(
        body["messages"][1]["content"],
        "data [QUJD], type [image/png], path []\n\nQ: what is it"
    );
}
