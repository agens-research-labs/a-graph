use crate::common::embeddings::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A real Ollama with `embeddinggemma`: one vector of 768 numbers per text; texts of one meaning are closer.
#[tokio::test]
#[ignore = "needs a local Ollama with embeddinggemma"]
async fn live_embeddinggemma() {
    let m = manager(&own_yml(here()));
    let texts = ["a cat sleeps on the sofa", "un gatto dorme sul divano", "quarterly tax declaration"].map(String::from);
    let v = m.embed("emb_gemma", &texts).await.unwrap();
    assert!(v.len() == 3 && v.iter().all(|x| x.len() == 768));
    let cos = |a: &[f32], b: &[f32]| {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        dot / (a.iter().map(|x| x * x).sum::<f32>().sqrt() * b.iter().map(|x| x * x).sum::<f32>().sqrt())
    };
    assert!(cos(&v[0], &v[1]) > cos(&v[0], &v[2]));
}
