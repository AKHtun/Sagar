//! Synthetic EPUB + local HTTP provider coverage. No book content leaves loopback.
use base64::Engine;
use sagar_lib::{
    epub,
    services::{
        self,
        ai_selection::{self, AppendInput, PrepareInput, SelectionEvent},
    },
    storage::{self, ai_selection as repo},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};
use zip::{ZipWriter, write::SimpleFileOptions};

const SOURCE: &str = include_str!("../../tests/fixtures/ai/selected-content.xhtml");
fn fixture(dir: &Path, body: &str) -> std::path::PathBuf {
    let path = dir.join("source.epub");
    let png=base64::engine::general_purpose::STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAusB9Wl6n3sAAAAASUVORK5CYII=").expect("png");
    let files:Vec<(&str,Vec<u8>)>=vec![
        ("mimetype",b"application/epub+zip".to_vec()),
        ("META-INF/container.xml",br#"<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.to_vec()),
        ("OEBPS/content.opf",br#"<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="uid">urn:uuid:selection-fixture</dc:identifier><dc:title>Selection fixture</dc:title><dc:language>en</dc:language></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="image" href="images/river.png" media-type="image/png"/></manifest><spine><itemref idref="chapter"/></spine></package>"#.to_vec()),
        ("OEBPS/nav.xhtml",br#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml">Chapter</a></li></ol></nav></body></html>"#.to_vec()),
        ("OEBPS/chapter.xhtml",body.as_bytes().to_vec()),("OEBPS/images/river.png",png),
    ];
    let mut zip = ZipWriter::new(std::fs::File::create(&path).expect("file"));
    for (index, (name, bytes)) in files.iter().enumerate() {
        zip.start_file(
            *name,
            SimpleFileOptions::default().compression_method(if index == 0 {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            }),
        )
        .expect("entry");
        zip.write_all(bytes).expect("bytes");
    }
    zip.finish().expect("archive");
    path
}

struct MockProvider {
    stop: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<String>>>,
    fail_at: Arc<AtomicUsize>,
    delay: Arc<AtomicUsize>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl MockProvider {
    fn new(data: &Path) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        listener.set_nonblocking(true).expect("nonblocking");
        let conn = services::workspace::open_db(data).expect("db");
        storage::ai_providers::create_custom(
            &conn,
            &storage::ai_providers::NewCustomProvider {
                id: "test".into(),
                name: "Loopback".into(),
                base_url: format!("http://{}/v1", listener.local_addr().expect("address")),
                api_key: "private-test-key".into(),
                model: "m".into(),
                created_at: "2026-10-15T00:00:00Z".into(),
            },
        )
        .expect("provider");
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(vec![]));
        let fail_at = Arc::new(AtomicUsize::new(usize::MAX));
        let delay = Arc::new(AtomicUsize::new(0));
        let (s, r, f, d) = (
            stop.clone(),
            requests.clone(),
            fail_at.clone(),
            delay.clone(),
        );
        let thread = std::thread::spawn(move || {
            'accept: while !s.load(Ordering::SeqCst) {
                let (mut socket, _) = match listener.accept() {
                    Ok(v) => v,
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                };
                socket.set_nonblocking(false).expect("blocking socket");
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("timeout");
                let mut bytes = vec![];
                let mut buf = [0; 4096];
                loop {
                    let count = match socket.read(&mut buf) {
                        Ok(n) => n,
                        Err(_) => continue 'accept,
                    };
                    if count == 0 {
                        continue 'accept;
                    }
                    bytes.extend_from_slice(&buf[..count]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let len = headers
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + len {
                            break;
                        }
                    }
                }
                let request = String::from_utf8(bytes).expect("utf8");
                let number = {
                    let mut rs = r.lock().expect("requests");
                    rs.push(request);
                    rs.len()
                };
                std::thread::sleep(Duration::from_millis(d.load(Ordering::SeqCst) as u64));
                let answer = if number == f.load(Ordering::SeqCst) {
                    "<p>broken"
                } else {
                    "<p>မြန်မာ</p>"
                };
                let stream = format!(
                    "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
                    serde_json::json!({"choices":[{"index":0,"delta":{"content":answer},"finish_reason":null}]}),
                    serde_json::json!({"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]})
                );
                let _ = write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    stream.len(),
                    stream
                );
            }
        });
        Self {
            stop,
            requests,
            fail_at,
            delay,
            thread: Some(thread),
        }
    }
}
impl Drop for MockProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("mock exit");
        }
    }
}

fn input(chapter: &str, ids: Vec<String>) -> PrepareInput {
    PrepareInput {
        chapter_id: chapter.into(),
        unit_ids: ids,
        agent: "custom:test".into(),
        model: "m".into(),
        reasoning_effort: None,
        current_instructions: "Keep the tone.".into(),
    }
}
fn execute(data: &Path, id: &str, action: &str, consent: bool) -> SelectionEvent {
    let (tx, rx) = mpsc::channel();
    let job = ai_selection::control(data, id, action, consent, move |e| {
        let _ = tx.send(e);
    })
    .expect("start");
    loop {
        let event = rx
            .recv_timeout(Duration::from_secs(15))
            .expect("terminal event");
        assert_eq!(event.job_id, job);
        if matches!(event.kind.as_str(), "done" | "failed" | "paused") {
            if event.kind == "failed" {
                eprintln!("Run failure: {}", event.message);
            }
            return event;
        }
    }
}
fn append_input(chapter: &str, ids: Vec<String>, revision: i64) -> AppendInput {
    AppendInput {
        chapter_id: chapter.into(),
        proposal_ids: ids,
        expected_revision: revision,
        request_id: uuid::Uuid::new_v4().to_string(),
        allow_again: false,
        confirm_out_of_order: false,
    }
}

#[test]
fn selected_content_translates_then_appends_exports_and_reopens_with_original_image_bytes() {
    let dir = tempfile::tempdir().expect("dir");
    let source = fixture(dir.path(), SOURCE);
    let before = std::fs::read(&source).expect("before");
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let mock = MockProvider::new(&data);
    let state = ai_selection::load(&data, chapter).expect("load");
    assert_eq!(state.units.len(), 10);
    assert!(
        state.units.iter().all(|u| u.unavailable.is_none()),
        "{:?}",
        state
            .units
            .iter()
            .map(|u| &u.unavailable)
            .collect::<Vec<_>>()
    );
    let ids = state.units.iter().map(|u| u.id.clone()).collect();
    let run = ai_selection::prepare(&data, input(chapter, ids)).expect("prepare");
    assert_eq!(execute(&data, &run, "start", true).kind, "done");
    let state = ai_selection::load(&data, chapter).expect("reload");
    let ready = &state.runs[0];
    assert!(ready.items.iter().all(|i| i.state == "ready"));
    assert!(ready.items.iter().any(|i| {
        i.preview_html
            .as_ref()
            .is_some_and(|h| h.contains("epubres://"))
    }));
    let conn = services::workspace::open_db(&data).expect("db");
    assert!(
        storage::documents::load(&conn, chapter)
            .expect("document")
            .html
            .is_none()
    );
    let run_json: String = conn
        .query_row(
            "SELECT config FROM ai_selection_runs WHERE id=?1",
            [&run],
            |r| r.get(0),
        )
        .expect("config");
    assert!(!run_json.contains("private-test-key"));
    let request_ids = ready.items.iter().map(|i| i.id.clone()).collect::<Vec<_>>();
    let mut args = append_input(chapter, request_ids, 0);
    let receipt = args.request_id.clone();
    let result = ai_selection::append(&data, args).expect("append");
    assert_eq!(result.revision, 1);
    assert_eq!(result.status, "draft");
    let html = result.html.expect("html");
    assert_eq!(html.matches("<img ").count(), 4);
    assert!(html.contains("rowspan=\"2\""));
    assert!(html.contains("<figcaption>"));
    assert!(!html.contains("data:image"));
    assert!(html.contains("မြန်မာ"));
    args = append_input(
        chapter,
        ready.items.iter().map(|i| i.id.clone()).collect(),
        0,
    );
    args.request_id = receipt;
    assert_eq!(
        ai_selection::append(&data, args)
            .expect("idempotent replay")
            .revision,
        1
    );
    let duplicate = ai_selection::append(
        &data,
        append_input(
            chapter,
            ready.items.iter().map(|i| i.id.clone()).collect(),
            1,
        ),
    )
    .err()
    .expect("duplicate rejected");
    assert_eq!(duplicate.code, "already_added");
    let destination = dir.path().join("translated.epub");
    services::export::export_epub(
        &data,
        services::export::ExportRequest {
            project_id: imported.project.id.clone(),
            destination: destination.clone(),
            mode: "draft".into(),
            title: None,
            language: None,
            include_original: true,
        },
    )
    .expect("export");
    let mut original =
        epub::archive::SafeArchive::open(&source, &Default::default()).expect("original archive");
    let mut output = epub::archive::SafeArchive::open(&destination, &Default::default())
        .expect("output archive");
    assert_eq!(
        original
            .read_entry("OEBPS/images/river.png")
            .expect("original image"),
        output
            .read_entry("OEBPS/images/river.png")
            .expect("exported image")
    );
    let exported = String::from_utf8(output.read_entry("OEBPS/chapter.xhtml").expect("chapter"))
        .expect("text");
    assert!(exported.contains("images/river.png"));
    assert!(exported.contains("<thead>"));
    let reopened = services::import::import_epub(&dir.path().join("reopened"), &destination, false)
        .expect("reimport");
    assert_eq!(reopened.chapters.len(), 1);
    assert_eq!(before, std::fs::read(&source).expect("unchanged original"));
    assert_eq!(
        before,
        std::fs::read(
            services::workspace::snapshot_path(&data, &imported.project.id).expect("snapshot")
        )
        .expect("immutable snapshot")
    );
    let stored = ai_selection::load(&data, chapter).expect("restart state");
    assert!(stored.units.iter().all(|u| u.added));
    for request in mock.requests.lock().expect("requests").iter() {
        assert!(!request.contains("data:image"));
        assert!(!request.contains("images/river.png"));
        assert!(!request.contains("alt="));
    }
}

#[test]
fn image_only_selection_needs_no_provider_or_consent_and_preserves_occurrences() {
    let dir = tempfile::tempdir().expect("dir");
    let source = fixture(
        dir.path(),
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Images</title></head><body><img src=\"images/river.png\" alt=\"First\"/><img src=\"images/river.png\" alt=\"Second\"/></body></html>",
    );
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let state = ai_selection::load(&data, chapter).expect("state");
    assert_eq!(state.units.len(), 2);
    assert_ne!(state.units[0].id, state.units[1].id);
    let mut args = input(chapter, state.units.iter().map(|u| u.id.clone()).collect());
    args.agent = String::new();
    args.model = String::new();
    let run = ai_selection::prepare(&data, args).expect("prepare images");
    assert_eq!(execute(&data, &run, "start", false).kind, "done");
    let next = ai_selection::load(&data, chapter).expect("next");
    let proposals = next.runs[0].items.iter().map(|i| i.id.clone()).collect();
    let result =
        ai_selection::append(&data, append_input(chapter, proposals, 0)).expect("append images");
    let html = result.html.expect("html");
    assert_eq!(html.matches("<img ").count(), 2);
    assert!(html.contains("alt=\"First\""));
    assert!(html.contains("alt=\"Second\""));
}

#[test]
fn failed_batch_pauses_and_smaller_retry_preserves_checkpoints_and_paragraph_boundary() {
    let dir = tempfile::tempdir().expect("dir");
    let long = "A long sentence to translate. မြန်မာစာ။ ".repeat(600);
    let source = fixture(
        dir.path(),
        &format!(
            "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Long</title></head><body><p id=\"long\">{long}</p><p>Later item.</p></body></html>"
        ),
    );
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let mock = MockProvider::new(&data);
    mock.fail_at.store(2, Ordering::SeqCst);
    let state = ai_selection::load(&data, chapter).expect("state");
    let run = ai_selection::prepare(
        &data,
        input(chapter, state.units.iter().map(|u| u.id.clone()).collect()),
    )
    .expect("prepare");
    assert_eq!(execute(&data, &run, "start", true).kind, "failed");
    let paused = ai_selection::load(&data, chapter).expect("paused");
    assert_eq!(paused.runs[0].items[0].parts_done, 1);
    assert_eq!(paused.runs[0].items[1].state, "pending");
    assert!(paused.runs[0].items[0].failed_output.is_some());
    assert!(paused.runs[0].items[0].preview_html.is_none());
    let checkpoint_conn = services::workspace::open_db(&data).expect("checkpoint DB");
    let checkpoint_item = repo::items(&checkpoint_conn, &run)
        .expect("items")
        .remove(0);
    let checkpoint = repo::parts(&checkpoint_conn, &checkpoint_item.id)
        .expect("parts")
        .remove(0);
    // Simulate reopening after an interrupted process. No worker may resume
    // merely because persisted state still said running.
    repo::set_run(&checkpoint_conn, &run, "running", None).expect("interruption marker");
    let recovered = ai_selection::load(&data, chapter).expect("recover");
    assert_eq!(recovered.runs[0].state, "paused");
    assert_eq!(recovered.runs[0].items[0].parts_done, 1);
    assert_eq!(execute(&data, &run, "smaller", true).kind, "done");
    let finished = ai_selection::load(&data, chapter).expect("finished");
    assert_eq!(finished.runs[0].budget, 2048);
    assert!(finished.runs[0].items.iter().all(|i| i.state == "ready"));
    let conn = services::workspace::open_db(&data).expect("conn");
    let item = repo::items(&conn, &run).expect("items").remove(0);
    let parts = repo::parts(&conn, &item.id).expect("parts");
    assert_eq!(parts[0].start, 0);
    assert_eq!(parts[0].id, checkpoint.id);
    assert_eq!(parts[0].end, checkpoint.end);
    assert_eq!(parts[0].output, checkpoint.output);
    assert_eq!(
        conn.query_row(
            "SELECT attempts FROM ai_selection_parts WHERE id=?1",
            [&checkpoint.id],
            |r| r.get::<_, i64>(0)
        )
        .expect("attempt count"),
        1
    );
    assert!(item.proposal.expect("proposal").matches("<p ").count() == 1);
    assert_eq!(
        mock.requests.lock().expect("requests").len(),
        parts.len() + 2
    ); // one failed attempt, plus later item
}

#[test]
fn cancel_and_skip_keep_completed_work_and_revision_conflicts_do_not_create_receipts() {
    let dir = tempfile::tempdir().expect("dir");
    let source = fixture(
        dir.path(),
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>T</title></head><body><p>One.</p><p>Two.</p><p>Three.</p></body></html>",
    );
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let mock = MockProvider::new(&data);
    mock.fail_at.store(2, Ordering::SeqCst);
    let state = ai_selection::load(&data, chapter).expect("state");
    let run = ai_selection::prepare(
        &data,
        input(chapter, state.units.iter().map(|u| u.id.clone()).collect()),
    )
    .expect("prepare");
    assert_eq!(execute(&data, &run, "start", true).kind, "failed");
    assert_eq!(execute(&data, &run, "skip", true).kind, "done");
    let completed = ai_selection::load(&data, chapter).expect("done");
    assert_eq!(completed.runs[0].items[1].state, "skipped");
    assert_eq!(mock.requests.lock().expect("requests").len(), 3);
    let conn = services::workspace::open_db(&data).expect("db");
    storage::documents::save(
        &conn,
        chapter,
        Some("<p>Manual writing.</p>".into()),
        "draft",
        0,
    )
    .expect("manual save");
    let proposals: Vec<_> = completed.runs[0]
        .items
        .iter()
        .filter(|i| i.state == "ready")
        .map(|i| i.id.clone())
        .collect();
    let err = ai_selection::append(&data, append_input(chapter, proposals.clone(), 0))
        .err()
        .expect("conflict");
    assert_eq!(err.code, "revision_conflict");
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM ai_selection_appends", [], |r| r
            .get::<_, i64>(0))
            .expect("receipts"),
        0
    );
    let saved = ai_selection::append(&data, append_input(chapter, proposals, 1))
        .expect("append after flush");
    assert!(
        saved
            .html
            .expect("html")
            .starts_with("<p>Manual writing.</p>")
    );
    mock.delay.store(400, Ordering::SeqCst);
    let fresh = ai_selection::prepare(&data, input(chapter, vec![state.units[0].id.clone()]))
        .expect("new run");
    let (tx, rx) = mpsc::channel();
    ai_selection::control(&data, &fresh, "start", true, move |e| {
        let _ = tx.send(e);
    })
    .expect("start");
    std::thread::sleep(Duration::from_millis(50));
    ai_selection::control(&data, &fresh, "cancel", false, |_| {}).expect("cancel");
    loop {
        let event = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("cancel event");
        if event.kind == "paused" {
            break;
        }
    }
    assert_eq!(
        ai_selection::load(&data, chapter)
            .expect("cancel state")
            .runs[0]
            .state,
        "paused"
    );
}

#[test]
fn missing_unsafe_and_unsupported_images_are_unavailable_without_network_requests() {
    let dir = tempfile::tempdir().expect("dir");
    let source = fixture(
        dir.path(),
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>T</title></head><body><img src=\"images/missing.png\"/><p><img src=\"https://invalid.example/image.png\"/></p><svg xmlns=\"http://www.w3.org/2000/svg\"><image href=\"images/river.png\"/></svg></body></html>",
    );
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let state = ai_selection::load(&data, chapter).expect("state");
    assert_eq!(state.units.len(), 3);
    assert!(state.units.iter().all(|u| u.unavailable.is_some()));
    assert!(
        ai_selection::prepare(
            &data,
            input(chapter, state.units.iter().map(|u| u.id.clone()).collect())
        )
        .is_err()
    );
}

#[test]
fn out_of_order_and_duplicate_anchor_checks_leave_drafts_and_receipts_unchanged() {
    let dir = tempfile::tempdir().expect("dir");
    let source = fixture(
        dir.path(),
        "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>T</title></head><body><p id=\"first\">First.</p><p id=\"last\">Last.</p></body></html>",
    );
    let data = dir.path().join("data");
    let imported = services::import::import_epub(&data, &source, false).expect("import");
    let chapter = &imported.chapters[0].id;
    let _mock = MockProvider::new(&data);
    let initial = ai_selection::load(&data, chapter).expect("source");
    let again = ai_selection::load(&data, chapter).expect("reopen source");
    assert_eq!(
        initial.units.iter().map(|u| &u.id).collect::<Vec<_>>(),
        again.units.iter().map(|u| &u.id).collect::<Vec<_>>()
    );
    let run = ai_selection::prepare(
        &data,
        input(
            chapter,
            initial.units.iter().map(|u| u.id.clone()).collect(),
        ),
    )
    .expect("prepare");
    assert!(ai_selection::control(&data, &run, "start", false, |_| {}).is_err());
    assert_eq!(execute(&data, &run, "start", true).kind, "done");
    let state = ai_selection::load(&data, chapter).expect("results");
    let items = &state.runs[0].items;
    ai_selection::append(&data, append_input(chapter, vec![items[1].id.clone()], 0))
        .expect("later first");
    let err = ai_selection::append(&data, append_input(chapter, vec![items[0].id.clone()], 1))
        .err()
        .expect("order warning");
    assert_eq!(err.code, "append_out_of_order");
    let mut earlier = append_input(chapter, vec![items[0].id.clone()], 1);
    earlier.confirm_out_of_order = true;
    let saved = ai_selection::append(&data, earlier).expect("explicit order consent");
    let html = saved.html.expect("html");
    assert!(html.find("id=\"last\"").expect("last") < html.find("id=\"first\"").expect("first"));
    let mut repeated = append_input(chapter, vec![items[1].id.clone()], 2);
    repeated.allow_again = true;
    repeated.confirm_out_of_order = true;
    assert_eq!(
        ai_selection::append(&data, repeated)
            .err()
            .expect("duplicate anchor")
            .code,
        "duplicate_id"
    );
    let conn = services::workspace::open_db(&data).expect("db");
    let unchanged = storage::documents::load(&conn, chapter).expect("unchanged");
    assert_eq!(unchanged.revision, 2);
    assert_eq!(unchanged.html.as_deref(), Some(html.as_str()));
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM ai_selection_appends", [], |r| r
            .get::<_, i64>(0))
            .expect("receipts"),
        2
    );
}
