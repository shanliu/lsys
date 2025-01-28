use std::{collections::HashMap, vec};

use crate::AreaResult;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use tantivy::query::BooleanQuery;
use tantivy::query::Query;
use tantivy::query::TermQuery;
use tantivy::query_grammar::Occur;
use tantivy::tokenizer::WhitespaceTokenizer;

use tantivy::tokenizer::PreTokenizedString;
use unicode_segmentation::UnicodeSegmentation;

use super::utils::key_word_clear;
use tantivy::schema::*;
use tantivy::{collector::TopDocs, IndexWriter};
use tantivy::{doc, Index};

#[derive(Debug)]
pub struct AreaCodeItem {
    pub code: String,
    pub name: String,
    pub leaf: bool,
}

#[derive(Debug)]
pub struct AreaCodeRelatedItem {
    pub selected: bool,
    pub item: AreaCodeItem,
}

#[derive(Debug)]
pub struct AreaSearchItem {
    pub item: Vec<AreaCodeItem>,
    pub source: f32,
}

#[derive(Debug)]
pub struct AreaCodeData {
    pub code: String,     //编码
    pub hide: bool,       //是否隐藏
    pub name: String,     //名称
    pub key_word: String, //关键字
}

#[derive(Debug, Clone)]
pub struct AreaCodeIndexInfo {
    pub hide: bool,   //是否隐藏
    pub name: String, //名称
}

pub trait AreaCodeIndexData {
    fn clear(&mut self) -> AreaResult<()>;
    fn get(&self, key: &str) -> Option<AreaCodeIndexInfo>;
    fn set(&mut self, key: &str, val: AreaCodeIndexInfo) -> AreaResult<()>;
    fn save(&mut self, version: &str) -> AreaResult<()>;
    fn version(&self) -> String;
    fn init(&mut self) -> AreaResult<()>;
}

pub trait AreaCodeIndexTree {
    fn clear(&mut self) -> AreaResult<()>;
    fn add(&mut self, code_data: Vec<&str>) -> AreaResult<()>;
    fn childs(&self, code_data: &[&str]) -> Option<Vec<(String, bool)>>;
    fn version(&self) -> String;
    fn save(&mut self, version: &str) -> AreaResult<()>;
    fn init(&mut self) -> AreaResult<()>;
}

pub trait AreaCodeTantivy {
    fn create_index(&self, schema: Schema) -> AreaResult<Index>;
    fn index_writer(&self, index: &Index) -> AreaResult<IndexWriter>;
}

pub trait AreaCodeProvider {
    type CD: AreaCodeIndexData + Sync;
    type CT: AreaCodeIndexTree + Sync;
    type TT: AreaCodeTantivy + Sync;
}

pub struct AreaCode<AP: AreaCodeProvider> {
    code_tantivy: AP::TT,
    code_data: AP::CD,
    code_data_tree: AP::CT,
    tantivy_index: Index,
    tantivy_keyword_field: Field,
    tantivy_code_field: Field,
}

impl<AP: AreaCodeProvider> AreaCode<AP> {
    pub fn new(
        code_tantivy: AP::TT,
        mut code_data: AP::CD,
        mut code_data_tree: AP::CT,
    ) -> AreaResult<Self> {
        let mut schema_builder = Schema::builder();
        let code_index = "code";
        let tantivy_code_field = schema_builder.add_text_field(code_index, STORED);

        let keyword_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("keyword")
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        let keyword_index = "keyword";
        let tantivy_keyword_field = schema_builder.add_text_field(keyword_index, keyword_options);
        let schema = schema_builder.build();
        let tantivy_index = code_tantivy.create_index(schema)?;
        tantivy_index
            .tokenizers()
            .register("keyword", WhitespaceTokenizer::default());
        code_data_tree.init()?;
        code_data.init()?;
        Ok(Self {
            code_tantivy,
            code_data,
            code_data_tree,
            tantivy_index,
            tantivy_code_field,
            tantivy_keyword_field,
        })
    }
    pub fn version_match(&self, version: &str) -> bool {
        self.code_data.version().as_str() == version
            && self.code_data_tree.version().as_str() == version
    }
    pub fn load_data(
        &mut self,
        area_code_data: Vec<AreaCodeData>,
        version: &str,
    ) -> AreaResult<()> {
        let mut index_writer = self.code_tantivy.index_writer(&self.tantivy_index)?;
        let code_data = &mut self.code_data;
        let code_data_tree = &mut self.code_data_tree;

        code_data.clear()?;
        code_data_tree.clear()?;
        let mut key_words = HashMap::with_capacity(area_code_data.len());
        for tmp_area in area_code_data.iter() {
            let code_name = tmp_area.name.as_str();
            code_data.set(
                &tmp_area.code,
                AreaCodeIndexInfo {
                    hide: tmp_area.hide,
                    name: code_name.to_owned(),
                },
            )?;
            code_data_tree.add(Self::code_parse(&tmp_area.code))?;
            key_words.insert(tmp_area.code.as_str(), &tmp_area.key_word);
        }
        code_data.save(version)?;
        code_data_tree.save(version)?;
        for (code, last_key_word) in key_words.iter() {
            let mut area_doc = doc!();
            let mut codes = Self::code_parse(code);
            let mut kws = Vec::with_capacity(codes.len());
            codes.pop();
            for item in codes.into_iter() {
                if let Some(pre_keyword) = &key_words.get(item) {
                    kws.push(pre_keyword.to_string());
                }
            }
            kws.push(last_key_word.to_string());
            // println!("{}", kws.join(" "));
            area_doc.add_field_value(self.tantivy_keyword_field, kws.join(" "));
            //如果搜索关键字为空,不加索引
            area_doc.add_field_value(
                self.tantivy_code_field,
                PreTokenizedString {
                    tokens: vec![],
                    text: code.to_owned().to_owned(),
                },
            );
            index_writer.add_document(area_doc)?;
        }
        index_writer.commit()?;
        Ok(())
    }

    pub(crate) fn code_parse(code: &str) -> Vec<&str> {
        let code = code.trim();
        if code.is_empty() {
            return vec![];
        }
        let len = code.len();
        let mut search_code = vec![];
        let mut start = 0;
        while len > start {
            if start < 6 {
                let end = start + 2;
                search_code.push(&code[0..if start + 2 < len { end } else { len }]);
                start = end;
            } else if start < 9 {
                let end = start + 3;
                search_code.push(&code[0..if start + 3 < len { end } else { len }]);
                start = end;
            } else {
                search_code.push(code);
                break;
            }
        }
        search_code
    }
    /// 列出指定行政区域编码下的可用区域
    pub fn childs(&self, code: &str) -> AreaResult<Vec<AreaCodeItem>> {
        let code_data = Self::code_parse(code);
        Ok(self
            .code_data_tree
            .childs(&code_data)
            .unwrap_or_default()
            .iter()
            .flat_map(|(last_code, next)| match self.code_data.get(last_code) {
                Some(tmp) => {
                    if tmp.hide {
                        None
                    } else {
                        Some(AreaCodeItem {
                            code: last_code.to_string(),
                            name: tmp.name,
                            leaf: !next,
                        })
                    }
                }
                None => Some(AreaCodeItem {
                    code: last_code.to_string(),
                    name: "[-.-]".to_string(),
                    leaf: !next,
                }),
            })
            .collect::<Vec<_>>())
    }
    /// 通过行政区域编码解析出区域
    pub fn find(&self, code: &str) -> AreaResult<Vec<AreaCodeItem>> {
        let code_data = Self::code_parse(code);
        if code_data.is_empty() {
            return Ok(vec![]);
        }
        let mut last_path = Vec::with_capacity(code_data.len());
        let mut out = Vec::with_capacity(code_data.len());
        for tmp in code_data {
            if let Some(tmp_info) = self.code_data.get(tmp) {
                if !tmp_info.hide {
                    out.push(AreaCodeItem {
                        code: tmp.to_owned(),
                        name: tmp_info.name.to_owned(),
                        leaf: false,
                    })
                }
                last_path.push(tmp);
            };
        }
        if let Some(e) = out.last_mut() {
            e.leaf = self
                .code_data_tree
                .childs(&last_path)
                .map(|e| e.is_empty())
                .unwrap_or(true);
        }
        Ok(out)
    }
    /// 获取行政区域编码同级区域的相关数据
    pub fn related(&self, code: &str) -> AreaResult<Vec<Vec<AreaCodeRelatedItem>>> {
        let code_data = Self::code_parse(code);
        let mut out_list = Vec::with_capacity(5);
        let mut now_list = Some(self.childs("")?);
        for ddd in code_data {
            let mut end = false;
            if let Some(tmp) = now_list {
                out_list.push(
                    tmp.into_iter()
                        .map(|e| {
                            let selected = if e.code == *ddd {
                                end = e.leaf;
                                true
                            } else {
                                false
                            };
                            AreaCodeRelatedItem { selected, item: e }
                        })
                        .collect::<Vec<_>>(),
                );
            }
            if end {
                now_list = None;
                break;
            }
            now_list = Some(self.childs(ddd)?);
        }
        if let Some(tmp) = now_list {
            out_list.push(
                tmp.into_iter()
                    .map(|e| AreaCodeRelatedItem {
                        selected: false,
                        item: e,
                    })
                    .collect::<Vec<_>>(),
            );
        }
        Ok(out_list)
    }

    /// 通过部分地址获取可能区域
    pub fn search(&self, name: &str, limit: usize) -> AreaResult<Vec<AreaSearchItem>> {
        if name.trim().is_empty() {
            return Ok(vec![]);
        }
        let mut search_data = vec![];
        for tmp in name.unicode_words() {
            let tmp = key_word_clear(tmp);
            if tmp.is_empty() {
                continue;
            }
            let key_str = tmp.to_lowercase();
            //  println!("{:?}", key_str);
            let term_query: Box<dyn Query> = Box::new(TermQuery::new(
                Term::from_field_text(self.tantivy_keyword_field, &key_str),
                IndexRecordOption::Basic,
            ));
            search_data.push((Occur::Must, term_query));
        }
        let query = BooleanQuery::new(search_data);

        let reader = self.tantivy_index.reader()?;
        let searcher = reader.searcher();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        //  println!("{:?}", top_docs);
        let out = top_docs
            .into_par_iter()
            .flat_map(|(source, doc_address)| {
                if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                    if let Some(code) = retrieved_doc.get_first(self.tantivy_code_field) {
                        if let Some(code) = code.as_str() {
                            if let Ok(item) = self.find(code) {
                                if !item.is_empty() {
                                    return Some(AreaSearchItem { item, source });
                                }
                            }
                        }
                    }
                };
                None
            })
            .collect();
        Ok(out)
    }
}
