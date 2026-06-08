use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path as AxumPath, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tokio::fs;
use tokio::net::TcpListener;
use uuid::Uuid;

use agentready_core::job::{self, ConvertOptions, JobProgress};
use agentready_core::models::JobResult;
use agentready_core::text_quality;

const MAX_FILES: usize = 25;
const MAX_FILE_SIZE_MB: u64 = 50;
// Axum defaults to 2 MB — raise to match V1 limits (25 × 50 MB + small overhead).
const MAX_UPLOAD_BODY_BYTES: usize =
    (MAX_FILES as u64 * MAX_FILE_SIZE_MB * 1024 * 1024 + 4 * 1024 * 1024) as usize;

#[derive(Clone)]
struct AppState {
    jobs: Arc<RwLock<HashMap<String, Arc<JobHandle>>>>,
}

struct JobHandle {
    status: RwLock<JobStatus>,
    base_dir: PathBuf,
    input_dir: PathBuf,
    output_dir: PathBuf,
}

#[derive(Clone)]
enum JobStatus {
    Converting {
        progress: JobProgress,
    },
    Completed {
        result: JobResult,
    },
    Failed {
        message: String,
    },
}

pub fn run(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_async(host, port).await })
}

async fn run_async(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = AppState {
        jobs: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(index_page))
        .route("/health", get(health))
        .route("/api/jobs", post(create_job))
        .route("/api/jobs/{job_id}", get(get_job).delete(delete_job))
        .route("/api/jobs/{job_id}/download", get(download_zip))
        .route("/api/jobs/{job_id}/preview", get(preview_file))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BODY_BYTES))
        .with_state(state);

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            return Err(format!(
                "Port {port} is already in use on {host}. \
                 Stop the other process (lsof -i :{port}) or run: \
                 agentready serve --port {}",
                port + 1
            )
            .into());
        }
        Err(e) => return Err(e.into()),
    };

    let url = format!("http://{addr}");
    eprintln!();
    eprintln!("  AgentReady UI running at {url}");
    eprintln!("  Health check: {url}/health");
    eprintln!("  Press Ctrl+C to stop.");
    eprintln!();
    tracing::info!("AgentReady UI at {url}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

async fn index_page() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn create_job(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let job_id = format!("job-{}", Uuid::new_v4());
    let base_dir = std::env::temp_dir().join(format!("agentready-{job_id}"));
    let input_dir = base_dir.join("input");
    let output_dir = base_dir.join("output");

    fs::create_dir_all(&input_dir).await?;

    let mut file_count = 0usize;
    let mut skipped_empty = 0usize;
    let mut skipped_type = 0usize;
    let mut skipped_large = 0usize;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let raw_name = field.file_name().unwrap_or("upload.bin").to_string();
        let safe_name = job::sanitize_filename(&raw_name);
        let ext = Path::new(&safe_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !matches!(ext.as_str(), "txt" | "md" | "csv" | "docx" | "pdf" | "epub" | "mobi") {
            skipped_type += 1;
            continue;
        }

        if file_count >= MAX_FILES {
            continue;
        }

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::bad_request(e.to_string()))?;

        if bytes.is_empty() {
            skipped_empty += 1;
            continue;
        }

        if bytes.len() as u64 > MAX_FILE_SIZE_MB * 1024 * 1024 {
            skipped_large += 1;
            continue;
        }

        let path = input_dir.join(&safe_name);
        fs::write(&path, &bytes).await?;
        file_count += 1;
    }

    if file_count == 0 {
        let _ = fs::remove_dir_all(&base_dir).await;
        let message = if skipped_empty > 0 {
            "Uploaded file(s) were empty (0 bytes). If dragging from iCloud Drive, open the file in Preview first or copy it to Desktop, then upload again.".to_string()
        } else if skipped_large > 0 {
            format!("File(s) exceed the {MAX_FILE_SIZE_MB} MB limit.")
        } else if skipped_type > 0 {
            "Unsupported file type. Use TXT, Markdown, CSV, DOCX, PDF, EPUB, or MOBI.".to_string()
        } else {
            "No supported files uploaded.".to_string()
        };
        return Err(AppError::bad_request(message));
    }

    let handle = Arc::new(JobHandle {
        status: RwLock::new(JobStatus::Converting {
            progress: JobProgress {
                done: 0,
                total: file_count,
                current_file: None,
            },
        }),
        base_dir: base_dir.clone(),
        input_dir: input_dir.clone(),
        output_dir: output_dir.clone(),
    });

    state
        .jobs
        .write()
        .unwrap()
        .insert(job_id.clone(), handle.clone());

    let job_id_spawn = job_id.clone();
    tokio::task::spawn_blocking(move || run_job_blocking(handle));

    Ok(Json(serde_json::json!({
        "id": job_id_spawn,
        "status": "Converting"
    })))
}

fn run_job_blocking(handle: Arc<JobHandle>) {
    let options = ConvertOptions {
        max_files: MAX_FILES,
        max_file_size_bytes: MAX_FILE_SIZE_MB * 1024 * 1024,
        recursive: false,
        zip_name: None,
    };

    let mut progress_fn = |p: JobProgress| {
        if let Ok(mut status) = handle.status.write() {
            *status = JobStatus::Converting { progress: p };
        }
    };

    let result = job::run_job(
        &[handle.input_dir.clone()],
        &handle.output_dir,
        &options,
        Some(&mut progress_fn),
    );

    match result {
        Ok(job_result) => {
            *handle.status.write().unwrap() = JobStatus::Completed { result: job_result };
        }
        Err(e) => {
            *handle.status.write().unwrap() = JobStatus::Failed {
                message: e.message(),
            };
        }
    }
}

async fn get_job(
    State(state): State<AppState>,
    AxumPath(job_id): AxumPath<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let jobs = state.jobs.read().unwrap();
    let handle = jobs
        .get(&job_id)
        .ok_or_else(|| AppError::not_found("Job not found"))?;

    let status = handle.status.read().unwrap().clone();
    let body = match status {
        JobStatus::Converting { progress } => serde_json::json!({
            "id": job_id,
            "status": "Converting",
            "progress": {
                "done": progress.done,
                "total": progress.total,
                "current_file": progress.current_file,
            },
            "result": null
        }),
        JobStatus::Completed { result } => serde_json::json!({
            "id": job_id,
            "status": "Completed",
            "result": result,
        }),
        JobStatus::Failed { message } => serde_json::json!({
            "id": job_id,
            "status": "Failed",
            "error": message,
            "result": null
        }),
    };

    Ok(Json(body))
}

async fn download_zip(
    State(state): State<AppState>,
    AxumPath(job_id): AxumPath<String>,
) -> Result<(StatusCode, [(header::HeaderName, HeaderValue); 2], Vec<u8>), AppError> {
    let zip_path = {
        let jobs = state.jobs.read().unwrap();
        let handle = jobs
            .get(&job_id)
            .ok_or_else(|| AppError::not_found("Job not found"))?;
        match handle.status.read().unwrap().clone() {
            JobStatus::Completed { result } => PathBuf::from(result.export_zip),
            _ => return Err(AppError::not_found("Job not ready")),
        }
    };

    let bytes = fs::read(&zip_path)
        .await
        .map_err(|_| AppError::not_found("Zip missing"))?;

    let disposition = HeaderValue::from_str(&format!(
        "attachment; filename=\"AgentReady-Export-{job_id}.zip\""
    ))
    .map_err(|e| AppError::bad_request(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("application/zip")),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    ))
}

#[derive(Deserialize)]
struct PreviewQuery {
    path: String,
}

async fn preview_file(
    State(state): State<AppState>,
    AxumPath(job_id): AxumPath<String>,
    Query(query): Query<PreviewQuery>,
) -> Result<(StatusCode, [(header::HeaderName, HeaderValue); 1], String), AppError> {
    let output_folder = {
        let jobs = state.jobs.read().unwrap();
        let handle = jobs
            .get(&job_id)
            .ok_or_else(|| AppError::not_found("Job not found"))?;
        match handle.status.read().unwrap().clone() {
            JobStatus::Completed { result } => PathBuf::from(result.output_folder),
            _ => return Err(AppError::not_found("Job not ready")),
        }
    };

    let rel = query.path.trim_start_matches('/');
    let md_path = output_folder.join(rel);

    if !md_path.starts_with(&output_folder) {
        return Err(AppError::bad_request("Invalid path".into()));
    }

    let content = fs::read_to_string(&md_path)
        .await
        .map_err(|_| AppError::not_found("Markdown file missing"))?;

    let body = if text_quality::looks_like_garbage(&content) {
        text_quality::GARBAGE_PREVIEW_MESSAGE.to_string()
    } else {
        content
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"))],
        body,
    ))
}

async fn delete_job(
    State(state): State<AppState>,
    AxumPath(job_id): AxumPath<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let handle = {
        let mut jobs = state.jobs.write().unwrap();
        jobs.remove(&job_id)
    };

    if let Some(handle) = handle {
        let _ = fs::remove_dir_all(&handle.base_dir).await;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }
}
