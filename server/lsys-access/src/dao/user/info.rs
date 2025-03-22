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

#[derive(Debug, Serialize)]
pub struct UserPublicInfo {
    pub id: u64,
    pub app_id: u64,
    pub user_data: String,
    pub user_name: String,
    pub user_account: String,
}
impl UserInfo {
    pub fn to_public(&self) -> UserPublicInfo {
        UserPublicInfo {
            id: self.id,
            app_id: self.app_id,
            user_data: self.user_data.clone(),
            user_name: hide_middle_two_chars_and_pad(&self.user_name),
            user_account: hide_middle_two_chars_and_pad(&self.user_account),
        }
    }
}

impl From<UserModel> for UserInfo {
    fn from(e: UserModel) -> Self {
        UserInfo {
            id: e.id,
            app_id: e.app_id,
            user_data: e.user_data.to_owned(),
            user_name: e.user_name.to_owned(),
            user_account: e.user_account.to_owned(),
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
    pub fn get(&self, user_id: &u64) -> Option<UserInfo> {
        self.data.get(user_id).map(|e| UserInfo::from(e.to_owned()))
    }
    pub fn into_array(self) -> Vec<UserModel> {
        self.data.into_iter().map(|e| e.1).collect::<Vec<_>>()
    }
}
