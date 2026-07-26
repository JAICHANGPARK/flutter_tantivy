use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{BooleanQuery, QueryParser, RegexQuery};
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument, Term};

/// Flutter에서 사용할 문서 구조체
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub title: Option<String>,
    pub text: String,
}

/// Flutter에서 사용할 검색 결과 구조체
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub doc: Document,
    pub snippet: Option<String>,
}

/// Flutter에서 사용할 검색 응답 구조체 (검색 결과 목록 + 총 매칭 문서 수)
#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub total_hits: usize,
    pub results: Vec<SearchResult>,
}

/// Tantivy의 핵심 로직을 관리하는 구조체
struct TantivyApi {
    index: Index,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
    id_field: Field,
    title_field: Field,
    text_field: Field,
}

/// 전역 상태를 Lazy와 Arc<Mutex<...>>로 안전하게 관리
static STATE: Lazy<Arc<Mutex<Option<TantivyApi>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// Tantivy 인덱스를 초기화하는 함수 (`tokenizer_type`: "default" 또는 "cjk"/"ngram" 선택 가능)
#[flutter_rust_bridge::frb(sync)]
pub fn init_tantivy(dir_path: String, tokenizer_type: Option<String>) -> Result<()> {
    let mut state_lock = STATE.lock().unwrap();
    if state_lock.is_some() {
        return Ok(());
    }

    let index_dir = PathBuf::from(dir_path);
    std::fs::create_dir_all(&index_dir)?;

    let t_type = tokenizer_type.unwrap_or_else(|| "default".to_string());
    let use_cjk = t_type.to_lowercase() == "cjk" || t_type.to_lowercase() == "ngram";

    let (index, schema) = if index_dir.join("meta.json").exists() {
        let index = Index::open_in_dir(&index_dir)?;
        let schema = index.schema();
        (index, schema)
    } else {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED);

        if use_cjk {
            let text_indexing = TextFieldIndexing::default()
                .set_tokenizer("ngram")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions);
            let text_options = TextOptions::default()
                .set_indexing_options(text_indexing)
                .set_stored();

            schema_builder.add_text_field("title", text_options.clone());
            schema_builder.add_text_field("text", text_options);
        } else {
            schema_builder.add_text_field("title", TEXT | STORED);
            schema_builder.add_text_field("text", TEXT | STORED);
        }

        let schema = schema_builder.build();
        let index = Index::create_in_dir(&index_dir, schema)?;
        (index.clone(), index.schema())
    };

    if use_cjk {
        let ngram_tokenizer = NgramTokenizer::new(1, 3, false)?;
        let ngram_analyzer = TextAnalyzer::builder(ngram_tokenizer)
            .filter(LowerCaser)
            .build();
        index.tokenizers().register("ngram", ngram_analyzer);
    }

    let id_field = schema
        .get_field("id")
        .map_err(|_| anyhow!("'id' field not found"))?;
    let title_field = schema
        .get_field("title")
        .map_err(|_| anyhow!("'title' field not found"))?;
    let text_field = schema
        .get_field("text")
        .map_err(|_| anyhow!("'text' field not found"))?;

    let writer = index.writer(50_000_000)?; // 50MB heap

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    let api = TantivyApi {
        index,
        writer: Mutex::new(writer),
        reader,
        id_field,
        title_field,
        text_field,
    };

    *state_lock = Some(api);

    Ok(())
}

/// Tantivy 상태를 해제하고 인덱스 및 리소스를 닫는 함수
#[flutter_rust_bridge::frb(sync)]
pub fn close_tantivy() -> Result<()> {
    let mut state_lock = STATE.lock().unwrap();
    *state_lock = None;
    Ok(())
}

/// [CREATE/UPDATE] 단일 문서를 추가하는 함수 (즉시 commit)
pub fn add_document(doc: Document) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    let id_term = Term::from_field_text(api.id_field, &doc.id);
    writer.delete_term(id_term);

    let mut tantivy_doc = TantivyDocument::new();
    tantivy_doc.add_text(api.id_field, &doc.id);
    if let Some(ref title) = doc.title {
        tantivy_doc.add_text(api.title_field, title);
    }
    tantivy_doc.add_text(api.text_field, &doc.text);

    writer.add_document(tantivy_doc)?;
    writer.commit()?;

    Ok(())
}

/// [READ] 쿼리로 문서를 검색하는 함수 (페이지네이션, 하이라이팅 스니펫 및 총 결과 수 지원)
pub fn search_documents(
    query: String,
    top_k: usize,
    offset: Option<usize>,
    enable_snippet: Option<bool>,
) -> Result<SearchResponse> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let query_parser = QueryParser::for_index(&api.index, vec![api.title_field, api.text_field]);
    let parsed_query = query_parser.parse_query(&query)?;

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) =
        searcher.search(&parsed_query, &(top_docs_collector, Count))?;

    let snippet_generator = if enable_snippet.unwrap_or(false) {
        Some(SnippetGenerator::create(
            &searcher,
            &parsed_query,
            api.text_field,
        )?)
    } else {
        None
    };

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        let id = retrieved_doc
            .get_first(api.id_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let title = retrieved_doc
            .get_first(api.title_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let text = retrieved_doc
            .get_first(api.text_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let snippet = snippet_generator
            .as_ref()
            .map(|g| g.snippet_from_doc(&retrieved_doc).to_html());

        results.push(SearchResult {
            score,
            doc: Document { id, title, text },
            snippet,
        });
    }

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 정규표현식(Regex) 패턴으로 문서를 검색하는 함수
pub fn search_documents_regex(
    pattern: String,
    top_k: usize,
    offset: Option<usize>,
    enable_snippet: Option<bool>,
) -> Result<SearchResponse> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let text_regex = RegexQuery::from_pattern(&pattern, api.text_field)?;
    let title_regex = RegexQuery::from_pattern(&pattern, api.title_field)?;

    let query = BooleanQuery::union(vec![Box::new(text_regex), Box::new(title_regex)]);

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) = searcher.search(&query, &(top_docs_collector, Count))?;

    let snippet_generator = if enable_snippet.unwrap_or(false) {
        Some(SnippetGenerator::create(
            &searcher,
            &query,
            api.text_field,
        )?)
    } else {
        None
    };

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address)?;
        let id = retrieved_doc
            .get_first(api.id_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let title = retrieved_doc
            .get_first(api.title_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let text = retrieved_doc
            .get_first(api.text_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let snippet = snippet_generator
            .as_ref()
            .map(|g| g.snippet_from_doc(&retrieved_doc).to_html());

        results.push(SearchResult {
            score,
            doc: Document { id, title, text },
            snippet,
        });
    }

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] ID로 특정 문서를 가져오는 함수
#[flutter_rust_bridge::frb(sync)]
pub fn get_document_by_id(id: String) -> Result<Option<Document>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let searcher = api.reader.searcher();
    let id_term = Term::from_field_text(api.id_field, &id);
    let query = tantivy::query::TermQuery::new(id_term, IndexRecordOption::Basic);

    let top_docs = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;

    if let Some((_, doc_address)) = top_docs.first() {
        let retrieved_doc = searcher.doc::<TantivyDocument>(*doc_address)?;
        let title = retrieved_doc
            .get_first(api.title_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let text = retrieved_doc
            .get_first(api.text_field)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        return Ok(Some(Document { id, title, text }));
    }

    Ok(None)
}

/// [READ] 인덱스에 등록된 전체 문서 수를 가져오는 함수
#[flutter_rust_bridge::frb(sync)]
pub fn get_num_docs() -> Result<u64> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();
    Ok(searcher.num_docs())
}

/// [UPDATE] 문서를 업데이트하는 함수
pub fn update_document(doc: Document) -> Result<()> {
    add_document(doc)
}

/// [DELETE] 문서를 삭제하는 함수
pub fn delete_document(id: String) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    let id_term = Term::from_field_text(api.id_field, &id);

    writer.delete_term(id_term);
    writer.commit()?;

    Ok(())
}

/// [DELETE] 모든 문서를 일괄 삭제하는 함수
pub fn delete_all_documents() -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    writer.delete_all_documents()?;
    writer.commit()?;

    Ok(())
}

/// [BATCH] 여러 문서를 한 번에 추가하는 함수
pub fn add_documents_batch(docs: Vec<Document>) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    for doc in docs {
        let id_term = Term::from_field_text(api.id_field, &doc.id);
        writer.delete_term(id_term);

        let mut tantivy_doc = TantivyDocument::new();
        tantivy_doc.add_text(api.id_field, &doc.id);
        if let Some(ref title) = doc.title {
            tantivy_doc.add_text(api.title_field, title);
        }
        tantivy_doc.add_text(api.text_field, &doc.text);

        writer.add_document(tantivy_doc)?;
    }

    writer.commit()?;

    Ok(())
}

/// [BATCH] 여러 문서를 한 번에 삭제하는 함수
pub fn delete_documents_batch(ids: Vec<String>) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();

    for id in ids {
        let id_term = Term::from_field_text(api.id_field, &id);
        writer.delete_term(id_term);
    }

    writer.commit()?;

    Ok(())
}

/// [UTILITY] 명시적으로 commit을 수행하는 함수
#[flutter_rust_bridge::frb(sync)]
pub fn commit() -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let mut writer = api.writer.lock().unwrap();
    writer.commit()?;

    Ok(())
}

/// [CREATE] commit 없이 문서를 추가하는 함수
pub fn add_document_no_commit(doc: Document) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let writer = api.writer.lock().unwrap();

    let id_term = Term::from_field_text(api.id_field, &doc.id);
    writer.delete_term(id_term);

    let mut tantivy_doc = TantivyDocument::new();
    tantivy_doc.add_text(api.id_field, &doc.id);
    if let Some(ref title) = doc.title {
        tantivy_doc.add_text(api.title_field, title);
    }
    tantivy_doc.add_text(api.text_field, &doc.text);

    writer.add_document(tantivy_doc)?;

    Ok(())
}

/// [DELETE] commit 없이 문서를 삭제하는 함수
pub fn delete_document_no_commit(id: String) -> Result<()> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let writer = api.writer.lock().unwrap();
    let id_term = Term::from_field_text(api.id_field, &id);

    writer.delete_term(id_term);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tantivy_full_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        init_tantivy(dir_path, Some("cjk".into())).unwrap();

        // 1. Add single document with title (Korean/CJK text)
        let doc1 = Document {
            id: "1".into(),
            title: Some("플러터 전문 검색 엔진".into()),
            text: "Tantivy 기반의 고성능 Flutter 전문 검색 라이브러리입니다.".into(),
        };
        add_document(doc1).unwrap();
        assert_eq!(get_num_docs().unwrap(), 1);

        // 2. Get document by ID
        let fetched = get_document_by_id("1".into()).unwrap().unwrap();
        assert_eq!(fetched.title.as_deref(), Some("플러터 전문 검색 엔진"));

        // 3. Search documents (CJK tokenized search)
        let search_res = search_documents("검색".into(), 10, None, Some(true)).unwrap();
        assert_eq!(search_res.total_hits, 1);
        assert_eq!(search_res.results[0].doc.id, "1");

        // 4. Regex search
        let regex_res = search_documents_regex("고성.*".into(), 10, None, Some(false)).unwrap();
        assert_eq!(regex_res.total_hits, 1);

        // 5. Delete all documents & close
        delete_all_documents().unwrap();
        assert_eq!(get_num_docs().unwrap(), 0);

        close_tantivy().unwrap();
    }
}