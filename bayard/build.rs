fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/cluster.proto");
    println!("cargo:rerun-if-changed=proto/healthcheck.proto");
    println!("cargo:rerun-if-changed=proto/index.proto");

    let cfg = prost_build::Config::default();
    tonic_build::configure()
        .type_attribute(
            ".",
            "#[derive(Serialize, Deserialize)]\n#[serde(rename_all = \"snake_case\")]",
        )
        .field_attribute(
            "healthcheck.LivenessResponse.State.UNKNOWN",
            "#[serde(rename = \"unknown\")]",
        )
        .field_attribute(
            "healthcheck.LivenessResponse.State.ALIVE",
            "#[serde(rename = \"alive\")]",
        )
        .field_attribute(
            "healthcheck.LivenessResponse.State.DEAD",
            "#[serde(rename = \"dead\")]",
        )
        .field_attribute(
            "healthcheck.ReadinessResponse.State.UNKNOWN",
            "#[serde(rename = \"unknown\")]",
        )
        .field_attribute(
            "healthcheck.ReadinessResponse.State.READY",
            "#[serde(rename = \"ready\")]",
        )
        .field_attribute(
            "healthcheck.ReadinessResponse.State.NOT_READY",
            "#[serde(rename = \"not_ready\")]",
        )
        .field_attribute(
            "index.CollectionKind.UNKNOWN",
            "#[serde(rename = \"unknown\")]",
        )
        .field_attribute(
            "index.CollectionKind.COUNT_AND_TOP_DOCS",
            "#[serde(rename = \"count_and_top_docs\")]",
        )
        .field_attribute("index.CollectionKind.COUNT", "#[serde(rename = \"count\")]")
        .field_attribute(
            "index.CollectionKind.TOP_DOCS",
            "#[serde(rename = \"top_docs\")]",
        )
        .field_attribute("index.Query.Kind.UNKNOWN", "#[serde(rename = \"unknown\")]")
        .field_attribute("index.Query.Kind.ALL", "#[serde(rename = \"all\")]")
        .field_attribute("index.Query.Kind.BOOLEAN", "#[serde(rename = \"boolean\")]")
        .field_attribute("index.Query.Kind.BOOST", "#[serde(rename = \"boost\")]")
        .field_attribute(
            "index.Query.Kind.FUZZY_TERM",
            "#[serde(rename = \"fuzzy_term\")]",
        )
        .field_attribute("index.Query.Kind.PHRASE", "#[serde(rename = \"phrase\")]")
        .field_attribute(
            "index.Query.Kind.QUERY_STRING",
            "#[serde(rename = \"query_string\")]",
        )
        .field_attribute("index.Query.Kind.RANGE", "#[serde(rename = \"range\")]")
        .field_attribute("index.Query.Kind.REGEX", "#[serde(rename = \"regex\")]")
        .field_attribute("index.Query.Kind.TERM", "#[serde(rename = \"term\")]")
        .field_attribute("index.Sort.Order.UNKNOWN", "#[serde(rename = \"unknown\")]")
        .field_attribute("index.Sort.Order.ASC", "#[serde(rename = \"asc\")]")
        .field_attribute("index.Sort.Order.DESC", "#[serde(rename = \"desc\")]")
        .out_dir("src/proto")
        .compile_with_config(
            cfg,
            &[
                "./proto/cluster.proto",
                "./proto/healthcheck.proto",
                "./proto/index.proto",
            ],
            &["./proto"],
        )?;
    Ok(())
}
