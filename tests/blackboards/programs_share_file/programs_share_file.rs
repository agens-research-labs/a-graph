use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Managers built apart from the same yml (as the service does for each request, as two runs of the
/// command line do) write the same file at once: no write fails or is lost, and the header stays.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn programs_share_file() {
    let (_, path) = file_board(here(), "programs_share_file");
    let writes: Vec<_> = (0..30)
        .map(|i| {
            let m = manager(here(), "programs_share_file").unwrap();
            tokio::spawn(async move { m.write("notes", &format!("line{i}"), Place::Bottom).await.unwrap() })
        })
        .collect();
    for write in writes {
        write.await.unwrap();
    }
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(HEADER), "the header is gone:\n{text}");
    for i in 0..30 {
        assert!(text.contains(&format!("line{i}\n")), "line{i} is lost:\n{text}");
    }
    manager(here(), "programs_share_file").unwrap();
}
