use fluent::FluentResource;
use intl_memoizer::concurrent::IntlLangMemoizer;

use crate::FluentMessage;

use crate::FluentBundleError;
use std::{collections::HashMap, path::Path, str::FromStr, sync::Arc};
use tokio::io::AsyncReadExt;
use unic_langid::LanguageIdentifier;
pub struct FluentMgr {
    default_bundle: Arc<FluentBundle>,
    default_lang: &'static str,
    bundle_data: HashMap<String, Arc<FluentBundle>>,
}

impl FluentMgr {
    async fn init_read_file<P: AsRef<Path>>(file_path: P) -> Result<String, FluentBundleError> {
        let mut f = tokio::fs::File::open(file_path.as_ref())
            .await
            .map_err(|e| {
                FluentBundleError::System(format!(
                    "open file[{:?}] error:{}",
                    file_path.as_ref(),
                    e
                ))
            })?;
        let mut buffer = Vec::new();
        // read the whole file
        f.read_to_end(&mut buffer).await?;
        Ok(String::from_utf8(buffer).unwrap_or_default())
    }
    fn init_create_bundle(
        lang: LanguageIdentifier,
        flt_data: &[&str],
    ) -> Result<fluent::bundle::FluentBundle<FluentResource, IntlLangMemoizer>, FluentBundleError>
    {
        let mut bundle = fluent::bundle::FluentBundle::new_concurrent(vec![lang.clone()]);
        bundle.set_use_isolating(false);
        for flt_str in flt_data {
            if !flt_str.is_empty() {
                let res = FluentResource::try_new(flt_str.to_string()).map_err(|(_, t)| {
                    FluentBundleError::System(
                        t.into_iter()
                            .map(|e| e.to_string())
                            .collect::<Vec<String>>()
                            .join(";"),
                    )
                })?;
                if let Err(err) = bundle.add_resource(res) {
                    tracing::error!("fluent add res:{:?}", err);
                }
            }
        }
        Ok(bundle)
    }

    pub async fn new<P: AsRef<Path>>(
        path: P,                       //语言文件路径
        app_fluent: &str,              //公共语言文件
        crate_fluent: Option<&[&str]>, //指定crate数据,传NONE遍历文件夹产生
    ) -> Result<Self, FluentBundleError> {
        let mut fluents: HashMap<String, Arc<FluentBundle>> = HashMap::new();
        let path = path.as_ref();
        match tokio::fs::read_dir(path).await {
            Ok(mut dir) => {
                while let Some(entry) = dir.next_entry().await? {
                    let ftype = entry.file_type().await;
                    if ftype.is_err() || !ftype?.is_dir() {
                        continue;
                    }
                    let lang = entry.file_name();
                    #[allow(unused_assignments)]
                    let mut lang_den = LanguageIdentifier::default();
                    let lang_str = lang.clone().into_string().unwrap_or_default();
                    match LanguageIdentifier::from_str(lang_str.as_str()) {
                        Err(_) => continue,
                        Ok(_lang_den) => {
                            lang_den = _lang_den;
                        }
                    }
                    let dir_path = path.to_path_buf().join(lang);

                    let pub_file = dir_path.clone().join(format!("./{}.ftl", app_fluent));
                    let pub_str = if pub_file.is_file() {
                        Self::init_read_file(pub_file).await?
                    } else {
                        "".to_string()
                    };

                    let mut fluent_item = vec![];

                    match crate_fluent {
                        Some(fluent_name) => {
                            for item in fluent_name {
                                fluent_item.push((
                                    dir_path.clone().join(format!("./{}.ftl", item)),
                                    item.to_string(),
                                ));
                            }
                        }
                        None => {
                            let mut sdir = tokio::fs::read_dir(dir_path.as_path()).await?;
                            while let Some(fileentry) = sdir.next_entry().await? {
                                if !fileentry.file_type().await?.is_file() {
                                    continue;
                                }
                                let file_path = fileentry.path();

                                if file_path.as_path().extension().unwrap_or_default() != "ftl" {
                                    continue;
                                }
                                let file_name = if let Some(name) = file_path.as_path().file_stem()
                                {
                                    name.to_string_lossy().to_string()
                                } else {
                                    continue;
                                };
                                if file_name.as_str() == app_fluent {
                                    continue;
                                }
                                fluent_item.push((file_path, file_name));
                            }
                        }
                    }
                    let mut bundles = FluentBundle {
                        fluent_bundles: HashMap::new(),
                        default_bundle: Some(Self::init_create_bundle(
                            lang_den.clone(),
                            &[&pub_str],
                        )?),
                    };
                    for (file_path, file_name) in fluent_item {
                        let file_path = file_path.as_path();
                        let ftl_string = Self::init_read_file(file_path).await?;
                        let bundle =
                            Self::init_create_bundle(lang_den.clone(), &[&ftl_string, &pub_str])?;
                        bundles.fluent_bundles.insert(file_name, bundle);
                    }
                    fluents.insert(lang_str, Arc::new(bundles));
                }
            }
            Err(err) => {
                tracing::error!("fluent dir:{:?} on {:?}", err, path);
            }
        }
        Ok(FluentMgr {
            bundle_data: fluents,
            default_lang: "en-US",
            default_bundle: Arc::new(FluentBundle {
                fluent_bundles: HashMap::new(),
                default_bundle: None,
            }),
        })
    }
    pub fn locale(&self, lang: Option<&str>) -> Arc<FluentBundle> {
        match lang {
            Some(lang) => self.bundle_data.get(lang).unwrap_or_else(|| {
                self.bundle_data
                    .get(self.default_lang)
                    .unwrap_or(&self.default_bundle)
            }),
            None => &self.default_bundle,
        }
        .to_owned()
    }
}

pub struct FluentBundle {
    default_bundle: Option<fluent::bundle::FluentBundle<FluentResource, IntlLangMemoizer>>,
    fluent_bundles: HashMap<String, fluent::bundle::FluentBundle<FluentResource, IntlLangMemoizer>>,
}

impl FluentBundle {
    pub fn format_message(&self, message: &FluentMessage) -> String {
        let message_find =
            |fluent: &fluent::bundle::FluentBundle<FluentResource, IntlLangMemoizer>| {
                fluent.get_message(&message.id).and_then(|msg| {
                    msg.value().map(|pattern| {
                        let mut args: fluent::FluentArgs = fluent::FluentArgs::new();
                        for (k, v) in &message.data {
                            let tmp = match v {
                                crate::FluentData::Message(fmsg) => self.format_message(fmsg),
                                crate::FluentData::MessageVec(fmsg) => fmsg
                                    .iter()
                                    .map(|msg| self.format_message(msg))
                                    .collect::<Vec<_>>()
                                    .join(";"),
                                crate::FluentData::String(msg) => msg.to_owned(),
                            };
                            args.set(k, tmp);
                        }
                        let mut errors = vec![];
                        fluent
                            .format_pattern(pattern, Some(&args), &mut errors)
                            .to_string()
                    })
                })
            };
        self.fluent_bundles
            .get(&message.crate_name)
            .and_then(message_find)
            .or(self.default_bundle.as_ref().and_then(message_find))
            .unwrap_or_else(|| {
                if message.data.is_empty() {
                    message.id.to_string()
                } else {
                    message.default_format()
                }
            })
    }
}
