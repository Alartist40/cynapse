use cynapse_mini::dendrite::{Dendrite, NodeType};

#[test]
fn test_dendrite_upsert_and_links() {
    let g = Dendrite::new();

    g.upsert(
        "identity",
        "Identity",
        "You are Cynapse. Connect to [[synapses]] and manage [[memory]].",
        NodeType::Identity,
        None,
    );

    g.upsert(
        "synapses",
        "Synapses",
        "Plugins that extend capabilities. Like [[leafcutter]] for inference.",
        NodeType::Concept,
        None,
    );

    g.upsert(
        "leafcutter",
        "LeafcutterLLM",
        "CPU-optimized inference. Supports [[quantization]].",
        NodeType::Project,
        None,
    );

    // Check structure: identity, synapses, leafcutter, memory (placeholder), quantization (placeholder)
    assert_eq!(g.len(), 5);

    let lc = g.get("leafcutter").unwrap();
    assert!(lc.links.contains(&"quantization".to_string()));

    let syn = g.get("synapses").unwrap();
    assert!(syn.backlinks.contains(&"identity".to_string()));
    assert!(syn.links.contains(&"leafcutter".to_string()));
}

#[test]
fn test_dendrite_backlink_rewire() {
    let g = Dendrite::new();

    g.upsert("a", "A", "Links to [[b]]", NodeType::Concept, None);
    g.upsert("b", "B", "Links to [[c]]", NodeType::Concept, None);

    let b = g.get("b").unwrap();
    assert!(b.backlinks.contains(&"a".to_string()));

    // Update a to no longer link to b
    g.upsert("a", "A", "No longer links to b.", NodeType::Concept, None);

    let b = g.get("b").unwrap();
    assert!(!b.backlinks.contains(&"a".to_string()));
}

#[test]
fn test_dendrite_multi_hop() {
    let g = Dendrite::new();

    g.upsert("a", "A", "→ [[b]]", NodeType::Concept, None);
    g.upsert("b", "B", "→ [[c]]", NodeType::Concept, None);
    g.upsert("c", "C", "→ [[d]]", NodeType::Concept, None);
    g.upsert("d", "D", "terminal", NodeType::Concept, None);

    let n1 = g.neighbors("a");
    assert_eq!(n1.len(), 1);
    assert_eq!(n1[0].id, "b");

    let n2 = g.neighbors_2hop("a");
    let ids: Vec<_> = n2.iter().map(|n| n.id.clone()).collect();
    assert!(ids.contains(&"b".to_string()));
    assert!(ids.contains(&"c".to_string()));
    assert!(!ids.contains(&"d".to_string()));

    let n3 = g.neighbors_3hop("a");
    let ids: Vec<_> = n3.iter().map(|n| n.id.clone()).collect();
    assert!(ids.contains(&"d".to_string()));
}

#[test]
fn test_dendrite_search() {
    let g = Dendrite::new();

    g.upsert("rust", "Rust", "A systems language.", NodeType::Concept, Some(vec!["lang".into()]));
    g.upsert("go", "Go", "A Google language.", NodeType::Concept, Some(vec!["lang".into()]));

    let results = g.search("rust");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "rust");

    let tagged = g.by_tag("lang");
    assert_eq!(tagged.len(), 2);
}

#[test]
fn test_dendrite_delete() {
    let g = Dendrite::new();

    g.upsert("a", "A", "[[b]]", NodeType::Concept, None);
    g.upsert("b", "B", "content", NodeType::Concept, None);

    assert!(g.delete("a"));
    assert!(g.get("a").is_none());

    let b = g.get("b").unwrap();
    assert!(!b.backlinks.contains(&"a".to_string()));
}

#[test]
fn test_dendrite_context_relevance() {
    let g = Dendrite::new();

    g.upsert("identity", "Identity", "You are Cynapse Mini.", NodeType::Identity, None);
    g.upsert("rust", "Rust", "Systems programming language.", NodeType::Concept, None);
    g.upsert("cargo", "Cargo", "Rust package manager.", NodeType::Concept, None);

    let ctx = cynapse_mini::dendrite::DendriteContext::new(g, None);
    let prompt = ctx.build_prompt("Tell me about Rust", 4000);

    assert!(prompt.contains("Rust"));
    assert!(prompt.contains("Identity"));
}
