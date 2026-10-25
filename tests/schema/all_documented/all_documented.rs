use a_graph::schema::{DOCUMENT, Field, SERVICE, Shape};

/// Every key, every value of `provider` and every named shape has its description, in the graph
/// and in the file of the service.
#[test]
fn all_documented() {
    let mut missing = Vec::new();
    let mut named = Vec::new();
    fields("", DOCUMENT, &mut missing, &mut named);
    fields("service", SERVICE, &mut missing, &mut named);
    assert!(missing.is_empty(), "without a description: {missing:?}");
}

fn fields(path: &str, list: &[Field], missing: &mut Vec<String>, named: &mut Vec<&'static str>) {
    for f in list {
        let p = if path.is_empty() { f.name.to_string() } else { format!("{path}.{}", f.name) };
        if f.doc.is_empty() {
            missing.push(p.clone());
        }
        shape(&p, &f.shape, missing, named);
    }
}

fn shape(path: &str, s: &Shape, missing: &mut Vec<String>, named: &mut Vec<&'static str>) {
    match s {
        Shape::List(inner) | Shape::MapOf(inner) => shape(path, inner, missing, named),
        Shape::Map(list) => fields(path, list, missing, named),
        Shape::Either(alternatives) => alternatives.iter().for_each(|a| shape(path, a, missing, named)),
        Shape::Tagged { by, by_doc, common, variants } => {
            if by_doc.is_empty() {
                missing.push(format!("{path}.{by}"));
            }
            fields(path, common, missing, named);
            for v in *variants {
                if v.doc.is_empty() {
                    missing.push(format!("{path} with {by}: {}", v.name));
                }
                fields(path, v.fields, missing, named);
            }
        }
        Shape::Lazy { name, doc, get } if !named.contains(name) => {
            named.push(name);
            if doc.is_empty() {
                missing.push(name.to_string());
            }
            shape(name, get(), missing, named);
        }
        _ => {}
    }
}
