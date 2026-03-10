use crate::types::{AnalysisResult, ImplementationStatus};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{env, fs, process::Command};

// ─── API DTOs (specific to the Convex HTTP protocol) ─────────────────────────

#[derive(Debug, Deserialize)]
struct LastHashResponse {
    content_hash: String,
}

#[derive(Debug, Deserialize)]
struct IngestResponse {
    run_id: String,
    is_duplicate: bool,
}

/// Serialisable method entry sent to the API.
/// Mirrors MethodTracking but with `status` as a plain string for the wire format.
#[derive(Debug, Serialize)]
struct ApiMethod {
    method_name: String,
    status: String,
}

/// Serialisable class entry sent to the API.
#[derive(Debug, Serialize)]
struct ApiClass {
    class_name: String,
    class_type: String,
    percentage_implemented: f32,
    methods: Vec<ApiMethod>,
}

#[derive(Debug, Serialize)]
struct IngestBody {
    commit_sha: String,
    branch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pr_number: Option<u32>,
    mc_version: String,
    content_hash: String,
    classes: Vec<ApiClass>,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn curl_get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("curl").args(["-sf", url]).output()?;
    if !out.status.success() {
        return Err(format!("GET {} failed: {}", url, out.status).into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn curl_post_json(
    url: &str,
    bearer: &str,
    body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("curl")
        .args([
            "-sf",
            "-X",
            "POST",
            url,
            "-H",
            &format!("Authorization: Bearer {}", bearer),
            "-H",
            "Content-Type: application/json",
            "-d",
            body,
        ])
        .output()?;
    if !out.status.success() {
        return Err(format!(
            "POST {} failed: {}\n{}",
            url,
            out.status,
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

// ─── Entry point ──────────────────────────────────────────────────────────────

/// Reads `outputs/analysis.json` produced by the tracker and ingests it into
/// Convex. Called by `main` after the analysis phase completes.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Environment validation
    let convex_site_url = env::var("CONVEX_SITE_URL").expect("Missing env: CONVEX_SITE_URL");
    let github_sha = env::var("GITHUB_SHA").expect("Missing env: GITHUB_SHA");
    let github_ref_name = env::var("GITHUB_REF_NAME").expect("Missing env: GITHUB_REF_NAME");
    let mc_version = env::var("MC_VERSION").expect("Missing env: MC_VERSION");
    let ingest_password = env::var("INGEST_PASSWORD").expect("Missing env: INGEST_PASSWORD");
    let pr_number: Option<u32> = env::var("PR_NUMBER").ok().and_then(|v| v.parse().ok());

    // 2. Read tracker output & hash
    // `outputs/analysis.json` is written by main.rs relative to the project root.
    let raw = fs::read_to_string("outputs/analysis.json")?;
    let content_hash = to_hex(&Sha256::digest(raw.as_bytes()));
    let data: AnalysisResult = serde_json::from_str(&raw)?;

    // 3. Duplicate check
    let mut hash_url = format!("{}/last-hash?branch={}", convex_site_url, github_ref_name);
    if let Some(pr) = pr_number {
        hash_url.push_str(&format!("&pr_number={}", pr));
    }

    let hash_body = curl_get(&hash_url)?;
    let latest: Option<LastHashResponse> = serde_json::from_str(&hash_body)?;

    if latest.is_some_and(|l| l.content_hash == content_hash) {
        println!("No changes detected — skipping ingestion.");
        return Ok(());
    }

    // 4. Map codebase types → API DTOs and ingest
    let classes: Vec<ApiClass> = data
        .classes
        .into_iter()
        .map(|cls| ApiClass {
            class_name: cls.class_name,
            class_type: cls.class_type,
            percentage_implemented: cls.percentage_implemented,
            methods: cls
                .methods
                .into_iter()
                .map(|m| ApiMethod {
                    method_name: m.method_name,
                    status: match m.status {
                        ImplementationStatus::Implemented => "Implemented".to_string(),
                        ImplementationStatus::NotImplemented => "NotImplemented".to_string(),
                    },
                })
                .collect(),
        })
        .collect();

    let body = IngestBody {
        commit_sha: github_sha,
        branch: github_ref_name,
        pr_number,
        mc_version,
        content_hash,
        classes,
    };

    let json_body = serde_json::to_string(&body)?;
    let ingest_body = curl_post_json(
        &format!("{}/ingest", convex_site_url),
        &ingest_password,
        &json_body,
    )?;

    let result: IngestResponse = serde_json::from_str(&ingest_body)?;
    println!(
        "Ingested run {} (is_duplicate: {})",
        result.run_id, result.is_duplicate
    );

    Ok(())
}
