use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

// Flutter에서 사용할 문서 구조체
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub text: String,
}

// Flutter에서 사용할 검색 결과 구조체
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub doc: Document,
}

// Tantivy의 핵심 로직을 관리하는 구조체
struct TantivyApi {
    index: Index,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
    schema: Schema,
    id_field: Field,
    text_field: Field,
}

// 전역 상태를 Lazy와 Arc<Mutex<...>>로 안전하게 관리
static STATE: Lazy<Arc<Mutex<Option<TantivyApi>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

// Tantivy 인덱스를 초기화하는 함수
// 초기화는 빠른 작업이므로 sync로 처리
#[flutter_rust_bridge::frb(sync)]
pub fn init_tantivy(dir_path: String) -> Result<()> {
    let mut state_lock = STATE.lock().unwrap();
    if state_lock.is_some() {
        // 이미 초기화된 경우
        return Ok(());
    }

    let index_dir = PathBuf::from(dir_path);
    std::fs::create_dir_all(&index_dir)?;

    let (index, schema) = if index_dir.join("meta.json").exists() {
        // 기존 인덱스 열기
        let index = Index::open_in_dir(&index_dir)?;
        let schema = index.schema();
        (index, schema)
    } else {
        // 새 인덱스 생성
        let mut schema_builder = Schema::builder();
        // ID 필드는 고유 식별자로 사용되며, 검색 가능하고 저장됩니다.
        schema_builder.add_text_field("id", STRING | STORED);
        // Text 필드는 전문 검색을 위해 사용됩니다.
        schema_builder.add_text_field("text", TEXT | STORED);
        let schema = schema_builder.build();
        let index = Index::create_in_dir(&index_dir, schema.clone())?;
        (index, schema)
    };

    let id_field = schema.get_field("id").map_err(|_| anyhow!("'id' field not found"))?;
    let text_field = schema.get_field("text").map_err(|_| anyhow!("'text' field not found"))?;

    let writer = index.writer(50_000_000)?; // 50MB heap

    // Reader를 생성하고 OnCommit 정책으로 자동 리로드
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    let api = TantivyApi {
        index,
        writer: Mutex::new(writer),
        reader,
        schema,
        id_field,
        text_field,
    };

    *state_lock = Some(api);

    Ok(())
}

// [CREATE] 새 문서를 추가하는 함수
// 즉시 commit하므로 단일 문서 추가에 적합
// 대량 추가는 add_documents_batch 사용 권장
pub fn add_document(doc: Document) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    // 추가하기 전에 동일한 ID의 문서가 있다면 삭제 (Update-or-Insert)
    let id_term = Term::from_field_text(api.id_field, &doc.id);
    writer.delete_term(id_term.clone());

    let mut tantivy_doc = TantivyDocument::new();
    tantivy_doc.add_text(api.id_field, &doc.id);
    tantivy_doc.add_text(api.text_field, &doc.text);

    writer.add_document(tantivy_doc)?;
    writer.commit()?;

    Ok(())
}

// [READ] 쿼리로 문서를 검색하는 함수
pub fn search_documents(query: String, top_k: usize) -> Result<Vec<SearchResult>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    // reader를 리로드하여 최신 변경사항을 반영
    api.reader.reload()?;

    // 전역 reader 재사용
    let searcher = api.reader.searcher();

    let query_parser = QueryParser::for_index(&api.index, vec![api.text_field]);
    let query = query_parser.parse_query(&query)?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(top_k))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        let id = retrieved_doc.get_first(api.id_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let text = retrieved_doc.get_first(api.text_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        results.push(SearchResult {
            score,
            doc: Document { id, text },
        });
    }

    Ok(results)
}

// [READ] ID로 특정 문서를 가져오는 함수
// ID 조회는 비교적 빠른 작업이므로 sync로 처리
#[flutter_rust_bridge::frb(sync)]
pub fn get_document_by_id(id: String) -> Result<Option<Document>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    // 전역 reader 재사용
    let searcher = api.reader.searcher();

    let id_term = Term::from_field_text(api.id_field, &id);
    let query = tantivy::query::TermQuery::new(id_term, IndexRecordOption::Basic);

    let top_docs = searcher.search(&query, &TopDocs::with_limit(1))?;

    if let Some((_, doc_address)) = top_docs.first() {
        let retrieved_doc = searcher.doc::<TantivyDocument>(*doc_address)?;
        let text = retrieved_doc.get_first(api.text_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        return Ok(Some(Document { id, text }));
    }

    Ok(None)
}


// [UPDATE] 문서를 업데이트하는 함수
pub fn update_document(doc: Document) -> Result<()> {
    // add_document가 내부적으로 delete & add 로직을 수행하므로 그대로 호출
    add_document(doc)
}

// [DELETE] 문서를 삭제하는 함수
pub fn delete_document(id: String) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    let id_term = Term::from_field_text(api.id_field, &id);

    writer.delete_term(id_term);
    writer.commit()?;

    Ok(())
}

// [BATCH] 여러 문서를 한 번에 추가하는 함수 (성능 최적화)
pub fn add_documents_batch(docs: Vec<Document>) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    for doc in docs {
        // 기존 문서가 있다면 삭제 (Update-or-Insert)
        let id_term = Term::from_field_text(api.id_field, &doc.id);
        writer.delete_term(id_term);

        let mut tantivy_doc = TantivyDocument::new();
        tantivy_doc.add_text(api.id_field, &doc.id);
        tantivy_doc.add_text(api.text_field, &doc.text);

        writer.add_document(tantivy_doc)?;
    }

    // 모든 문서를 추가한 후 한 번만 commit
    writer.commit()?;

    Ok(())
}

// [BATCH] 여러 문서를 한 번에 삭제하는 함수 (성능 최적화)
pub fn delete_documents_batch(ids: Vec<String>) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    for id in ids {
        let id_term = Term::from_field_text(api.id_field, &id);
        writer.delete_term(id_term);
    }

    // 모든 삭제 작업 후 한 번만 commit
    writer.commit()?;

    Ok(())
}

// [UTILITY] 명시적으로 commit을 수행하는 함수
// add_document_no_commit과 함께 사용하여 수동으로 트랜잭션 제어
#[flutter_rust_bridge::frb(sync)]
pub fn commit() -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    writer.commit()?;

    Ok(())
}

// [CREATE] commit 없이 문서를 추가하는 함수 (고급 사용자용)
// 여러 작업을 수행한 후 commit()을 호출하여 성능 최적화
pub fn add_document_no_commit(doc: Document) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    let id_term = Term::from_field_text(api.id_field, &doc.id);
    writer.delete_term(id_term);

    let mut tantivy_doc = TantivyDocument::new();
    tantivy_doc.add_text(api.id_field, &doc.id);
    tantivy_doc.add_text(api.text_field, &doc.text);

    writer.add_document(tantivy_doc)?;

    Ok(())
}

// [DELETE] commit 없이 문서를 삭제하는 함수 (고급 사용자용)
pub fn delete_document_no_commit(id: String) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock.as_ref().ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    let id_term = Term::from_field_text(api.id_field, &id);

    writer.delete_term(id_term);

    Ok(())
}