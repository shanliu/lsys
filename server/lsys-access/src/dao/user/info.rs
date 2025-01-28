use serde::Serialize;
use std::collections::HashMap;

use crate::model::UserModel;

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: u64,
    pub app_id: u64,
    pub user_data: String,
    pub user_name: String,
    pub user_account: String,
}

impl UserInfo {
    pub fn from_user_model(e: UserModel, privacy: bool) -> Self {
        UserInfo {
            id: e.id,
            app_id: e.app_id,
            user_data: e.user_data,
            user_name: if privacy {
                hide_middle_two_chars_and_pad(&e.user_name)
            } else {
                e.user_name
            },
            user_account: if privacy {
                hide_middle_two_chars_and_pad(&e.user_account)
            } else {
                e.user_account
            },
        }
    }
}

fn hide_middle_two_chars_and_pad(input: &str) -> String {
    let mut result = String::with_capacity(4);
    result.push(input.chars().next().unwrap_or_default());
    result.push_str("**");
    result.push(input.chars().last().unwrap_or_default());
    result
}

pub struct UserInfoSet {
    data: HashMap<u64, UserModel>,
}
impl UserInfoSet {
    pub fn new(data: HashMap<u64, UserModel>) -> Self {
        Self { data }
    }
    pub fn get(&self, user_id: &u64, privacy: bool) -> Option<UserInfo> {
        self.data
            .get(user_id)
            .map(|e| UserInfo::from_user_model(e.to_owned(), privacy))
    }
    pub fn into_array(self) -> Vec<UserModel> {
        self.data.into_iter().map(|e| e.1).collect::<Vec<_>>()
    }
}
