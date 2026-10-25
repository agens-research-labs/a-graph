use crate::common::blackboards::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Parallel writes to one board do not lose text.
#[test]
fn parallel_writes() {
    let (m, _) = file_board(here(), "parallel_writes");
    std::thread::scope(|s| {
        for i in 0..8 {
            let m = &m;
            s.spawn(move || m.write("notes", &format!("line{i}"), Place::Bottom).unwrap());
        }
    });
    let text = m.read("notes").unwrap();
    for i in 0..8 {
        assert!(text.contains(&format!("line{i}")));
    }
}
