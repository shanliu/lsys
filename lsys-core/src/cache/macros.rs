#[macro_export]
macro_rules! impl_cache_fetch_one {
    ($fn:ident,$dao_field:ident,$cache_field:ident,$fetch_type:ty,$result:ty) => {
        pub async fn $fn(&self, id: &$fetch_type) -> $result {
            match self.$dao_field.$cache_field.get(&id).await {
                Some(data) => Ok(data),
                None => match self.$dao_field.$fn(&id).await {
                    Ok(data) => {
                        self.$dao_field
                            .$cache_field
                            .set(id.to_owned(), data.clone(), 0)
                            .await;
                        Ok(data)
                    }
                    Err(e) => Err(e),
                },
            }
        }
    };
}
#[macro_export]
macro_rules! impl_cache_fetch_vec {
    ($fn:ident,$dao_field:ident,$cache_field:ident,$fetch_type:ty,$result:ty) => {
        pub async fn $fn(&self, ids: &[$fetch_type]) -> $result {
            let mut get = vec![];
            let mut hash = std::collections::HashMap::with_capacity(ids.len());
            for id in ids {
                match self.$dao_field.$cache_field.get(id).await {
                    Some(data) => {
                        hash.entry(id.to_owned()).or_insert(data);
                    }
                    None => {
                        get.push(id.to_owned());
                    }
                }
            }
            if !get.is_empty() {
                match self.$dao_field.$fn(&get).await {
                    Ok(datas) => {
                        for (pk, data) in datas.into_iter() {
                            self.$dao_field
                                .$cache_field
                                .set(pk.clone(), data.clone(), 0)
                                .await;
                            hash.entry(pk.clone()).or_insert(data);
                        }
                    }
                    Err(err) => return Err(err),
                }
            }
            Ok(hash)
        }
    };
}
