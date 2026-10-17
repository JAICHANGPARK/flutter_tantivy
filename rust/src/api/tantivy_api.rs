use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{
    AllQuery, BooleanQuery, ExistsQuery, FuzzyTermQuery, MoreLikeThisQuery, Occur, Query,
    QueryParser, RegexPhraseQuery, RegexQuery, TermQuery,
};
use tantivy::schema::*;
use tantivy::snippet::{Snippet, SnippetGenerator};
use tantivy::tokenizer::{
    Language, LowerCaser, NgramTokenizer, RawTokenizer, RemoveLongFilter, SimpleTokenizer, Stemmer,
    TextAnalyzer, WhitespaceTokenizer,
};
use tantivy::{
    DocAddress, Index, IndexReader, IndexWriter, ReloadPolicy, Searcher, TantivyDocument, Term,
};

/// Flutter에서 사용할 문서 구조체
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub id: String,
    pub title: Option<String>,
    pub text: String,
}

/// 텍스트 내 바이트 오프셋 범위 구조체 (하이라이트 구간 및 스니펫 프래그먼트 범위)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

/// Flutter에서 사용할 검색 결과 구조체
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub doc: Document,
    pub snippet: Option<String>,
    pub title_snippet: Option<String>,
    pub fragment_range: Option<TextRange>,
    pub highlighted_ranges: Option<Vec<TextRange>>,
}

/// Flutter에서 사용할 검색 응답 구조체 (검색 결과 목록 + 총 매칭 문서 수)
#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub total_hits: usize,
    pub results: Vec<SearchResult>,
}

/// 관대한(Lenient) 쿼리 파싱 검색 응답 구조체 (검색 결과 + 파싱 경고/오류 목록)
#[derive(Debug, Clone)]
pub struct LenientSearchResponse {
    pub total_hits: usize,
    pub results: Vec<SearchResult>,
    pub errors: Vec<String>,
}

/// 인덱스 용어 사전(Term Dictionary) 자동완성/추천 항목 구조체
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermSuggestion {
    pub term: String,
    pub doc_freq: u32,
}

/// 인덱스 디스크/메모리 공간 사용량 통계 구조체 (`Searcher::space_usage` 포팅)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSpaceUsage {
    pub total_bytes: u64,
    pub num_docs: u64,
    pub num_segments: usize,
    pub store_bytes: u64,
    pub postings_bytes: u64,
    pub positions_bytes: u64,
    pub fast_fields_bytes: u64,
    pub termdict_bytes: u64,
    pub fieldnorms_bytes: u64,
    pub deletes_bytes: u64,
}

/// Tantivy의 핵심 로직을 관리하는 구조체
struct TantivyApi {
    dir_path: String,
    tokenizer_type: String,
    index: Index,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
    id_field: Field,
    title_field: Field,
    text_field: Field,
}

/// 전역 상태를 Lazy와 Arc<Mutex<...>>로 안전하게 관리
static STATE: Lazy<Arc<Mutex<Option<TantivyApi>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// 토크나이저 타입 문자열을 정규화하고 실제 등록될 토크나이저 이름을 반환
fn resolve_tokenizer_name(tokenizer_type: &str) -> (&'static str, Option<Language>) {
    let lower = tokenizer_type.trim().to_lowercase();
    match lower.as_str() {
        "cjk" | "ngram" => ("ngram", None),
        "whitespace" => ("whitespace", None),
        "raw" => ("raw", None),
        "stem" | "stemmer" | "en_stem" | "stem:english" | "stem_english" => {
            ("en_stem", Some(Language::English))
        }
        "stem:arabic" | "stem_arabic" => ("stem_arabic", Some(Language::Arabic)),
        "stem:danish" | "stem_danish" => ("stem_danish", Some(Language::Danish)),
        "stem:dutch" | "stem_dutch" => ("stem_dutch", Some(Language::Dutch)),
        "stem:finnish" | "stem_finnish" => ("stem_finnish", Some(Language::Finnish)),
        "stem:french" | "stem_french" => ("stem_french", Some(Language::French)),
        "stem:german" | "stem_german" => ("stem_german", Some(Language::German)),
        "stem:greek" | "stem_greek" => ("stem_greek", Some(Language::Greek)),
        "stem:hungarian" | "stem_hungarian" => ("stem_hungarian", Some(Language::Hungarian)),
        "stem:italian" | "stem_italian" => ("stem_italian", Some(Language::Italian)),
        "stem:norwegian" | "stem_norwegian" => ("stem_norwegian", Some(Language::Norwegian)),
        "stem:portuguese" | "stem_portuguese" => ("stem_portuguese", Some(Language::Portuguese)),
        "stem:romanian" | "stem_romanian" => ("stem_romanian", Some(Language::Romanian)),
        "stem:russian" | "stem_russian" => ("stem_russian", Some(Language::Russian)),
        "stem:spanish" | "stem_spanish" => ("stem_spanish", Some(Language::Spanish)),
        "stem:swedish" | "stem_swedish" => ("stem_swedish", Some(Language::Swedish)),
        "stem:tamil" | "stem_tamil" => ("stem_tamil", Some(Language::Tamil)),
        "stem:turkish" | "stem_turkish" => ("stem_turkish", Some(Language::Turkish)),
        _ => ("default", None),
    }
}

/// 인덱스에 커스텀 토크나이저(CJK N-gram, Whitespace, 18개 언어 Stemmer)를 모두 등록
fn register_custom_tokenizers(index: &Index) -> Result<()> {
    let ngram_tokenizer = NgramTokenizer::new(1, 3, false)?;
    let ngram_analyzer = TextAnalyzer::builder(ngram_tokenizer)
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("ngram", ngram_analyzer);

    let whitespace_analyzer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .build();
    index
        .tokenizers()
        .register("whitespace", whitespace_analyzer);

    index.tokenizers().register("raw", RawTokenizer::default());

    let stemmer_languages = [
        ("en_stem", Language::English),
        ("stem_english", Language::English),
        ("stem_arabic", Language::Arabic),
        ("stem_danish", Language::Danish),
        ("stem_dutch", Language::Dutch),
        ("stem_finnish", Language::Finnish),
        ("stem_french", Language::French),
        ("stem_german", Language::German),
        ("stem_greek", Language::Greek),
        ("stem_hungarian", Language::Hungarian),
        ("stem_italian", Language::Italian),
        ("stem_norwegian", Language::Norwegian),
        ("stem_portuguese", Language::Portuguese),
        ("stem_romanian", Language::Romanian),
        ("stem_russian", Language::Russian),
        ("stem_spanish", Language::Spanish),
        ("stem_swedish", Language::Swedish),
        ("stem_tamil", Language::Tamil),
        ("stem_turkish", Language::Turkish),
    ];

    for (name, lang) in stemmer_languages {
        let analyzer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(lang))
            .build();
        index.tokenizers().register(name, analyzer);
    }

    Ok(())
}

/// 스니펫 프래그먼트의 원본 텍스트 내 바이트 오프셋 범위를 계산 (Tantivy #3134 `Snippet::fragment_range` 포팅)
fn compute_fragment_range(text: &str, snippet: &Snippet) -> Option<TextRange> {
    let fragment = snippet.fragment();
    if fragment.is_empty() {
        return Some(TextRange { start: 0, end: 0 });
    }
    let normalized = text.trim();
    normalized.find(fragment).map(|start| TextRange {
        start,
        end: start + fragment.len(),
    })
}

/// 검색된 문서 목록과 쿼리로부터 SearchResult 벡터를 생성하는 내부 헬퍼
fn collect_search_results(
    searcher: &Searcher,
    api: &TantivyApi,
    query: &dyn Query,
    top_docs: Vec<(f32, DocAddress)>,
    enable_snippet: bool,
) -> Result<Vec<SearchResult>> {
    let text_snippet_gen = if enable_snippet {
        SnippetGenerator::create(searcher, query, api.text_field).ok()
    } else {
        None
    };

    let title_snippet_gen = if enable_snippet {
        SnippetGenerator::create(searcher, query, api.title_field).ok()
    } else {
        None
    };

    let mut results = Vec::with_capacity(top_docs.len());
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

        let (snippet, fragment_range, highlighted_ranges) =
            if let Some(ref gen) = text_snippet_gen {
                let s = gen.snippet_from_doc(&retrieved_doc);
                if s.is_empty() {
                    (Some(String::new()), Some(TextRange { start: 0, end: 0 }), Some(Vec::new()))
                } else {
                    let html = s.to_html();
                    let frag_range = compute_fragment_range(&text, &s);
                    let highlights = s
                        .highlighted()
                        .iter()
                        .map(|r| TextRange {
                            start: r.start,
                            end: r.end,
                        })
                        .collect();
                    (Some(html), frag_range, Some(highlights))
                }
            } else {
                (None, None, None)
            };

        let title_snippet = title_snippet_gen.as_ref().and_then(|gen| {
            let s = gen.snippet_from_doc(&retrieved_doc);
            if s.is_empty() {
                None
            } else {
                Some(s.to_html())
            }
        });

        results.push(SearchResult {
            score,
            doc: Document { id, title, text },
            snippet,
            title_snippet,
            fragment_range,
            highlighted_ranges,
        });
    }

    Ok(results)
}

/// `field:*` 형태의 Exists 쿼리 문법을 지원하는 전처리 헬퍼 (Tantivy #2974 포팅)
/// `title:*`, `text:*`, `id:*` 등의 토큰이 단독 또는 복합 쿼리에 포함된 경우 regex `field:/.+/`로 변환하거나 ExistsQuery를 구성합니다.
fn preprocess_exists_query_syntax(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    let chars: Vec<char> = query.chars().collect();
    let mut i = 0;
    let mut in_quotes = false;

    while i < chars.len() {
        if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') {
            in_quotes = !in_quotes;
            out.push(chars[i]);
            i += 1;
            continue;
        }

        if !in_quotes {
            let rem: String = chars[i..].iter().collect();
            let prev_is_boundary = i == 0
                || chars[i - 1].is_whitespace()
                || matches!(chars[i - 1], '(' | '+' | '-');

            if prev_is_boundary {
                let mut matched_field = None;
                for field in ["title", "text", "id"] {
                    let pattern = format!("{field}:*");
                    if let Some(after) = rem.strip_prefix(&pattern) {
                        let next_ch = after.chars().next();
                        let next_is_boundary = next_ch.is_none()
                            || next_ch.unwrap().is_whitespace()
                            || matches!(next_ch.unwrap(), ')' | '^');
                        if next_is_boundary {
                            matched_field = Some((field, pattern.len()));
                            break;
                        }
                    }
                }

                if let Some((field, pat_len)) = matched_field {
                    out.push_str(&format!("{field}:/.+/"));
                    i += pat_len;
                    continue;
                }
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

/// Tantivy 인덱스를 초기화하는 함수
/// `tokenizer_type`: `"default"`, `"cjk"`/`"ngram"`, `"whitespace"`, `"raw"`, `"en_stem"`/`"stemmer"` 또는 `"stem:<language>"` 선택 가능
#[flutter_rust_bridge::frb(sync)]
pub fn init_tantivy(dir_path: String, tokenizer_type: Option<String>) -> Result<()> {
    let mut state_lock = STATE.lock().unwrap();
    let t_type = tokenizer_type.unwrap_or_else(|| "default".to_string());

    if let Some(existing) = state_lock.as_ref() {
        if existing.dir_path == dir_path && existing.tokenizer_type == t_type {
            return Ok(());
        }
    }

    let index_dir = PathBuf::from(&dir_path);
    std::fs::create_dir_all(&index_dir)?;

    let (tokenizer_name, _) = resolve_tokenizer_name(&t_type);

    let (index, schema) = if index_dir.join("meta.json").exists() {
        let index = Index::open_in_dir(&index_dir)?;
        let schema = index.schema();
        (index, schema)
    } else {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("id", STRING | STORED | FAST);

        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer(tokenizer_name)
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_indexing)
            .set_stored()
            .set_fast(None);

        schema_builder.add_text_field("title", text_options.clone());
        schema_builder.add_text_field("text", text_options);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(&index_dir, schema)?;
        (index.clone(), index.schema())
    };

    register_custom_tokenizers(&index)?;

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
        dir_path,
        tokenizer_type: t_type,
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

/// [READ] 쿼리로 문서를 검색하는 함수 (페이지네이션, 하이라이팅 스니펫, `field:*` Exists 문법 및 총 결과 수 지원)
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

    let mut query_parser =
        QueryParser::for_index(&api.index, vec![api.title_field, api.text_field]);
    query_parser.allow_regexes();
    let effective_query = preprocess_exists_query_syntax(&query);
    let parsed_query = query_parser.parse_query(&effective_query)?;

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) =
        searcher.search(&parsed_query, &(top_docs_collector, Count))?;

    let results = collect_search_results(
        &searcher,
        api,
        &*parsed_query,
        top_docs,
        enable_snippet.unwrap_or(false),
    )?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 관대한(Lenient) 쿼리 파서로 문서를 검색하는 함수 (문법 오류가 있는 사용자 입력도 안전하게 처리하며 필드 부스트 및 기본 AND 모드 지원)
pub fn search_documents_lenient(
    query: String,
    top_k: usize,
    offset: Option<usize>,
    enable_snippet: Option<bool>,
    conjunction_by_default: Option<bool>,
    title_boost: Option<f32>,
) -> Result<LenientSearchResponse> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let mut query_parser =
        QueryParser::for_index(&api.index, vec![api.title_field, api.text_field]);
    query_parser.allow_regexes();
    if conjunction_by_default.unwrap_or(false) {
        query_parser.set_conjunction_by_default();
    }
    if let Some(boost) = title_boost {
        query_parser.set_field_boost(api.title_field, boost);
    }

    let effective_query = preprocess_exists_query_syntax(&query);
    let (parsed_query, parse_errors) = query_parser.parse_query_lenient(&effective_query);
    let errors = parse_errors.into_iter().map(|e| e.to_string()).collect();

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) =
        searcher.search(&parsed_query, &(top_docs_collector, Count))?;

    let results = collect_search_results(
        &searcher,
        api,
        &*parsed_query,
        top_docs,
        enable_snippet.unwrap_or(false),
    )?;

    Ok(LenientSearchResponse {
        total_hits,
        results,
        errors,
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

    let results = collect_search_results(
        &searcher,
        api,
        &query,
        top_docs,
        enable_snippet.unwrap_or(false),
    )?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 정규표현식 구문(Regex Phrase) 패턴 시퀀스와 슬롭(slop)으로 문서를 검색하는 함수 (Tantivy #3135, #3137, #3138 포팅)
pub fn search_documents_regex_phrase(
    patterns: Vec<String>,
    slop: Option<u32>,
    max_expansions: Option<u32>,
    top_k: usize,
    offset: Option<usize>,
    enable_snippet: Option<bool>,
) -> Result<SearchResponse> {
    if patterns.is_empty() {
        return Ok(SearchResponse {
            total_hits: 0,
            results: Vec::new(),
        });
    }
    if patterns.len() == 1 {
        return search_documents_regex(
            patterns.into_iter().next().unwrap(),
            top_k,
            offset,
            enable_snippet,
        );
    }

    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let is_ngram = resolve_tokenizer_name(&api.tokenizer_type).0 == "ngram";
    let slop_val = if is_ngram {
        slop.unwrap_or(16).max(16)
    } else {
        slop.unwrap_or(0)
    };

    let normalized_patterns: Vec<String> = patterns
        .into_iter()
        .map(|p| {
            let lower = p.to_lowercase();
            if is_ngram
                && lower.chars().count() > 3
                && !lower.contains(['.', '*', '+', '?', '[', ']', '(', ')', '|', '^', '$', '\\'])
            {
                lower.chars().take(3).collect()
            } else {
                lower
            }
        })
        .collect();

    let terms_with_offset: Vec<(usize, String)> =
        normalized_patterns.into_iter().enumerate().collect();

    let mut text_phrase = RegexPhraseQuery::new_with_offset_and_slop(
        api.text_field,
        terms_with_offset.clone(),
        slop_val,
    );
    let mut title_phrase = RegexPhraseQuery::new_with_offset_and_slop(
        api.title_field,
        terms_with_offset,
        slop_val,
    );

    if let Some(max_exp) = max_expansions {
        text_phrase.set_max_expansions(max_exp);
        title_phrase.set_max_expansions(max_exp);
    }

    let query = BooleanQuery::union(vec![Box::new(text_phrase), Box::new(title_phrase)]);

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) = searcher.search(&query, &(top_docs_collector, Count))?;

    let results = collect_search_results(
        &searcher,
        api,
        &query,
        top_docs,
        enable_snippet.unwrap_or(false),
    )?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 오타 허용(Fuzzy / Levenshtein Automaton 기반)으로 문서를 검색하는 함수 (Tantivy #3136 포팅)
pub fn search_documents_fuzzy(
    term: String,
    distance: Option<u8>,
    transposition_cost_one: Option<bool>,
    prefix: Option<bool>,
    top_k: usize,
    offset: Option<usize>,
    enable_snippet: Option<bool>,
) -> Result<SearchResponse> {
    let trimmed = term.trim();
    if trimmed.is_empty() {
        return Ok(SearchResponse {
            total_hits: 0,
            results: Vec::new(),
        });
    }

    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let dist = distance.unwrap_or(1).min(2);
    let trans_cost_one = transposition_cost_one.unwrap_or(true);
    let is_prefix = prefix.unwrap_or(false);
    let is_ngram = resolve_tokenizer_name(&api.tokenizer_type).0 == "ngram";

    let words: Vec<String> = trimmed
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();

    let mut candidate_terms: Vec<String> = Vec::new();
    for w in &words {
        candidate_terms.push(w.clone());
        if is_ngram {
            let chars: Vec<char> = w.chars().collect();
            if chars.len() > 3 {
                for window in chars.windows(3) {
                    candidate_terms.push(window.iter().collect());
                }
            }
        }
    }

    let mut subqueries: Vec<Box<dyn Query>> = Vec::with_capacity(candidate_terms.len() * 2);
    for w in &candidate_terms {
        let text_term = Term::from_field_text(api.text_field, w);
        let title_term = Term::from_field_text(api.title_field, w);
        let (text_fuzzy, title_fuzzy) = if is_prefix {
            (
                FuzzyTermQuery::new_prefix(text_term, dist, trans_cost_one),
                FuzzyTermQuery::new_prefix(title_term, dist, trans_cost_one),
            )
        } else {
            (
                FuzzyTermQuery::new(text_term, dist, trans_cost_one),
                FuzzyTermQuery::new(title_term, dist, trans_cost_one),
            )
        };
        subqueries.push(Box::new(text_fuzzy));
        subqueries.push(Box::new(title_fuzzy));
    }

    let query = BooleanQuery::union(subqueries);

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) = searcher.search(&query, &(top_docs_collector, Count))?;

    let results = collect_search_results(
        &searcher,
        api,
        &query,
        top_docs,
        enable_snippet.unwrap_or(false),
    )?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 특정 문서 ID와 유사한 문서를 검색하는 함수 (`MoreLikeThisQuery` 포팅)
#[allow(clippy::too_many_arguments)]
pub fn search_similar_documents(
    id: String,
    top_k: usize,
    min_doc_frequency: Option<u64>,
    max_doc_frequency: Option<u64>,
    min_term_frequency: Option<usize>,
    min_word_length: Option<usize>,
    max_word_length: Option<usize>,
    boost_factor: Option<f32>,
) -> Result<SearchResponse> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let id_term = Term::from_field_text(api.id_field, &id);
    let id_query = TermQuery::new(id_term.clone(), IndexRecordOption::Basic);
    let target_hits = searcher.search(&id_query, &TopDocs::with_limit(1).order_by_score())?;

    let Some((_, doc_address)) = target_hits.first() else {
        return Ok(SearchResponse {
            total_hits: 0,
            results: Vec::new(),
        });
    };

    let mut builder = MoreLikeThisQuery::builder()
        .with_min_doc_frequency(min_doc_frequency.unwrap_or(1))
        .with_min_term_frequency(min_term_frequency.unwrap_or(1));

    if let Some(max_df) = max_doc_frequency {
        builder = builder.with_max_doc_frequency(max_df);
    }
    if let Some(min_wl) = min_word_length {
        builder = builder.with_min_word_length(min_wl);
    }
    if let Some(max_wl) = max_word_length {
        builder = builder.with_max_word_length(max_wl);
    }
    if let Some(boost) = boost_factor {
        builder = builder.with_boost_factor(boost);
    }

    let mlt_query = builder.with_document(*doc_address);

    // Exclude the reference document itself from similar results
    let exclude_self = BooleanQuery::new(vec![
        (Occur::Must, Box::new(mlt_query)),
        (
            Occur::MustNot,
            Box::new(TermQuery::new(id_term, IndexRecordOption::Basic)),
        ),
    ]);

    let top_docs_collector = TopDocs::with_limit(top_k).order_by_score();
    let (top_docs, total_hits) =
        searcher.search(&exclude_self, &(top_docs_collector, Count))?;

    let results = collect_search_results(&searcher, api, &exclude_self, top_docs, false)?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 특정 필드에 값이 존재하는 문서를 검색하는 함수 (`ExistsQuery` / Tantivy #2974, #3068 포팅)
pub fn search_documents_exists(
    field_name: String,
    top_k: usize,
    offset: Option<usize>,
) -> Result<SearchResponse> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let schema = api.index.schema();
    let field = schema
        .get_field(&field_name)
        .map_err(|_| anyhow!("Field '{field_name}' not found in schema"))?;
    let field_entry = schema.get_field_entry(field);

    let query: Box<dyn Query> = if field_entry.is_fast() {
        Box::new(ExistsQuery::new(field_name, true))
    } else if field == api.id_field {
        Box::new(AllQuery)
    } else {
        Box::new(RegexQuery::from_pattern(".+", field)?)
    };

    let offset_val = offset.unwrap_or(0);
    let top_docs_collector = TopDocs::with_limit(top_k)
        .and_offset(offset_val)
        .order_by_score();

    let (top_docs, total_hits) = searcher.search(&*query, &(top_docs_collector, Count))?;
    let results = collect_search_results(&searcher, api, &*query, top_docs, false)?;

    Ok(SearchResponse {
        total_hits,
        results,
    })
}

/// [READ] 접두사(prefix)로 인덱스 용어 사전(Term Dictionary)을 스트리밍 조회하여 추천 검색어 및 문서 빈도를 반환하는 함수 (Tantivy #3155 포팅)
pub fn suggest_terms(
    prefix: String,
    limit: Option<usize>,
    include_title: Option<bool>,
) -> Result<Vec<TermSuggestion>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();

    let is_ngram = resolve_tokenizer_name(&api.tokenizer_type).0 == "ngram";
    let raw_prefix = prefix.trim().to_lowercase();
    let normalized_prefix: String = if is_ngram && raw_prefix.chars().count() > 3 {
        raw_prefix.chars().take(3).collect()
    } else {
        raw_prefix
    };
    let max_items = limit.unwrap_or(10);
    let mut fields = vec![api.text_field];
    if include_title.unwrap_or(true) {
        fields.push(api.title_field);
    }

    let mut term_freqs: BTreeMap<String, u32> = BTreeMap::new();

    for segment_reader in searcher.segment_readers() {
        for &field in &fields {
            let Ok(inverted_index) = segment_reader.inverted_index(field) else {
                continue;
            };
            let termdict = inverted_index.terms();
            let mut range_builder = termdict.range();
            if !normalized_prefix.is_empty() {
                range_builder = range_builder.ge(normalized_prefix.as_bytes());
            }
            let Ok(mut stream) = range_builder.into_stream() else {
                continue;
            };

            while stream.advance() {
                let Ok(term_str) = std::str::from_utf8(stream.key()) else {
                    continue;
                };
                if !normalized_prefix.is_empty() && !term_str.starts_with(&normalized_prefix) {
                    break;
                }
                let doc_freq = stream.value().doc_freq;
                *term_freqs.entry(term_str.to_string()).or_insert(0) += doc_freq;
            }
        }
    }

    let mut suggestions: Vec<TermSuggestion> = term_freqs
        .into_iter()
        .map(|(term, doc_freq)| TermSuggestion { term, doc_freq })
        .collect();

    suggestions.sort_by(|a, b| b.doc_freq.cmp(&a.doc_freq).then_with(|| a.term.cmp(&b.term)));
    suggestions.truncate(max_items);

    Ok(suggestions)
}

/// [READ] ID로 특정 문서를 가져오는 함수
#[flutter_rust_bridge::frb(sync)]
pub fn get_document_by_id(id: String) -> Result<Option<Document>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();
    let id_term = Term::from_field_text(api.id_field, &id);
    let query = TermQuery::new(id_term, IndexRecordOption::Basic);

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

/// [READ] 인덱스의 디스크/컴포넌트별 공간 사용량 통계를 가져오는 함수 (`Searcher::space_usage` 포팅)
#[flutter_rust_bridge::frb(sync)]
pub fn get_index_space_usage() -> Result<IndexSpaceUsage> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    api.reader.reload()?;
    let searcher = api.reader.searcher();
    let usage = searcher.space_usage()?;

    let mut store_bytes = 0u64;
    let mut postings_bytes = 0u64;
    let mut positions_bytes = 0u64;
    let mut fast_fields_bytes = 0u64;
    let mut termdict_bytes = 0u64;
    let mut fieldnorms_bytes = 0u64;
    let mut deletes_bytes = 0u64;

    for seg in usage.segments() {
        store_bytes += seg.store().total().get_bytes();
        postings_bytes += seg.postings().total().get_bytes();
        positions_bytes += seg.positions().total().get_bytes();
        fast_fields_bytes += seg.fast_fields().total().get_bytes();
        termdict_bytes += seg.termdict().total().get_bytes();
        fieldnorms_bytes += seg.fieldnorms().total().get_bytes();
        deletes_bytes += seg.deletes().get_bytes();
    }

    Ok(IndexSpaceUsage {
        total_bytes: usage.total().get_bytes(),
        num_docs: searcher.num_docs(),
        num_segments: usage.segments().len(),
        store_bytes,
        postings_bytes,
        positions_bytes,
        fast_fields_bytes,
        termdict_bytes,
        fieldnorms_bytes,
        deletes_bytes,
    })
}

/// [READ] 현재 인덱스 메타데이터에 포함된 세그먼트 파일 상대 경로 목록을 가져오는 함수 (Tantivy #3080 `IndexMeta::list_segment_files` 포팅)
#[flutter_rust_bridge::frb(sync)]
pub fn list_index_segment_files() -> Result<Vec<String>> {
    let state_lock = STATE.lock().unwrap();
    let api = state_lock
        .as_ref()
        .ok_or_else(|| anyhow!("Tantivy not initialized"))?;

    let meta = api.index.load_metas()?;
    let files: HashSet<PathBuf> = meta
        .segments
        .iter()
        .flat_map(|seg| seg.list_files())
        .collect();

    let mut file_names: Vec<String> = files
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    file_names.sort();

    Ok(file_names)
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
    fn test_tantivy_full_lifecycle_and_new_features() {
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

        // 3. Search documents (CJK tokenized search + snippet fragment range & highlighted ranges)
        let search_res = search_documents("검색".into(), 10, None, Some(true)).unwrap();
        assert_eq!(search_res.total_hits, 1);
        assert_eq!(search_res.results[0].doc.id, "1");
        assert!(search_res.results[0].snippet.is_some());
        assert!(search_res.results[0].fragment_range.is_some());
        assert!(search_res.results[0]
            .highlighted_ranges
            .as_ref()
            .is_some_and(|r| !r.is_empty()));

        // 4. Regex search
        let regex_res = search_documents_regex("고성.*".into(), 10, None, Some(false)).unwrap();
        assert_eq!(regex_res.total_hits, 1);

        // 5. Add English documents for Fuzzy, RegexPhrase, MoreLikeThis, Exists, Suggest, SpaceUsage
        add_documents_batch(vec![
            Document {
                id: "2".into(),
                title: Some("Rust Search Library".into()),
                text: "Alan just got a part time job building fast search engines in Rust.".into(),
            },
            Document {
                id: "3".into(),
                title: None,
                text: "Rust systems programming provides memory safety and fast search engines."
                    .into(),
            },
        ])
        .unwrap();
        assert_eq!(get_num_docs().unwrap(), 3);

        // 6. Exists query (`title:*` syntax & `search_documents_exists`)
        let exists_res = search_documents_exists("title".into(), 10, None).unwrap();
        assert_eq!(exists_res.total_hits, 2); // doc 1 and doc 2 have title, doc 3 has None

        let exists_syntax_res = search_documents("title:*".into(), 10, None, Some(false)).unwrap();
        assert_eq!(exists_syntax_res.total_hits, 2);

        // 7. Lenient search with malformed query syntax (unclosed quote)
        let lenient_res = search_documents_lenient(
            "\"Rust".into(),
            10,
            None,
            Some(true),
            Some(false),
            Some(2.0),
        )
        .unwrap();
        assert!(!lenient_res.errors.is_empty());

        // 8. Space usage and segment files
        let usage = get_index_space_usage().unwrap();
        assert_eq!(usage.num_docs, 3);
        assert!(usage.total_bytes > 0);
        assert!(usage.num_segments >= 1);

        let seg_files = list_index_segment_files().unwrap();
        assert!(!seg_files.is_empty());

        // 9. Term suggestions
        let suggestions = suggest_terms("rust".into(), Some(5), Some(true)).unwrap();
        assert!(!suggestions.is_empty());

        // 10. Delete all documents & close
        delete_all_documents().unwrap();
        assert_eq!(get_num_docs().unwrap(), 0);

        close_tantivy().unwrap();

        // 11. Test default/stemmer tokenizer with Fuzzy, RegexPhrase, and MoreLikeThis
        let temp_dir_en = tempfile::tempdir().unwrap();
        let dir_path_en = temp_dir_en.path().to_str().unwrap().to_string();
        init_tantivy(dir_path_en, Some("default".into())).unwrap();

        add_documents_batch(vec![
            Document {
                id: "10".into(),
                title: Some("Flutter Framework".into()),
                text: "Alan just got a part time job working with Flutter and Dart.".into(),
            },
            Document {
                id: "11".into(),
                title: Some("Flutter Toolkit".into()),
                text: "Building mobile apps with Flutter and Dart is productive and fast.".into(),
            },
            Document {
                id: "12".into(),
                title: Some("Database Systems".into()),
                text: "PostgreSQL is an advanced relational database system.".into(),
            },
        ])
        .unwrap();

        // Fuzzy search ("Fluter" -> matches "Flutter" with distance 1)
        let fuzzy_res = search_documents_fuzzy(
            "Fluter".into(),
            Some(1),
            Some(true),
            Some(false),
            10,
            None,
            Some(true),
        )
        .unwrap();
        assert_eq!(fuzzy_res.total_hits, 2);

        // Regex phrase search ("pa.*" followed by "time")
        let regex_phrase_res = search_documents_regex_phrase(
            vec!["pa.*".into(), "time".into()],
            Some(0),
            None,
            10,
            None,
            Some(false),
        )
        .unwrap();
        assert_eq!(regex_phrase_res.total_hits, 1);
        assert_eq!(regex_phrase_res.results[0].doc.id, "10");

        // Similar documents (MoreLikeThis for doc "10" should match doc "11", not "12" or "10")
        let similar_res =
            search_similar_documents("10".into(), 10, Some(1), None, Some(1), Some(2), None, None)
                .unwrap();
        assert!(similar_res.total_hits >= 1);
        assert_eq!(similar_res.results[0].doc.id, "11");

        close_tantivy().unwrap();
    }
}