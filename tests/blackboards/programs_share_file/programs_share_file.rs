use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Managers built apart from the same yml (as the service does for each request, as two runs of the
/// command line do) write the same file at once: no write fails or is lost, and the header stays.
#[test]
fn programs_share_file() {
    let (_, path) = file_board(here(), "programs_share_file");
    let managers: Vec<_> = (0..30)
        .map(|_| manager(here(), "programs_share_file").unwrap())
        .collect();
    std::thread::scope(|s| {
        for (i, m) in managers.iter().enumerate() {
            s.spawn(move || m.write("notes", &format!("line{i}"), Place::Bottom).unwrap());
        }
    });
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.starts_with(HEADER), "the header is gone:\n{text}");
    for i in 0..30 {
        assert!(text.contains(&format!("line{i}\n")), "line{i} is lost:\n{text}");
    }
    manager(here(), "programs_share_file").unwrap();
}
