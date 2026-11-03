use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Parallel writes to one board do not lose text.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn parallel_writes() {
    let (m, _) = file_board(here(), "parallel_writes");
    let m = std::sync::Arc::new(m);
    let writes: Vec<_> = (0..8)
        .map(|i| {
            let m = m.clone();
            tokio::spawn(async move { m.write("notes", &format!("line{i}"), Place::Bottom).await.unwrap() })
        })
        .collect();
    for write in writes {
        write.await.unwrap();
    }
    let text = m.read("notes").unwrap();
    for i in 0..8 {
        assert!(text.contains(&format!("line{i}")));
    }
}
