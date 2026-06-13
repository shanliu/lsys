use std::ops::Deref;

use actix_utils::future::{Ready, err, ok};
use actix_web::{FromRequest, HttpRequest, dev::Payload, web::Data};

use lsys_web::lsys_app::dao::{RestAuthSession, RestAuthToken};
use lsys_web::lsys_user::dao::{UserAuthSession, UserAuthToken};
use lsys_web::{
    common::{JsonData, JsonResponse, RequestAuthDao, RestAuthQueryDao, UserAuthQueryDao},
    dao::WebDao,
};

use super::ResponseJson;

//正常用户登陆，如cookie登陆

pub struct UserAuthQuery {
    inner: UserAuthQueryDao,
}

impl Deref for UserAuthQuery {
    type Target = UserAuthQueryDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for UserAuthQuery {
    type Error = ResponseJson;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user_dao_opt = req.app_data::<Data<WebDao>>();
        match user_dao_opt {
            Some(app_dao) => {
                ok(Self {
                    inner: RequestAuthDao::new(
                        UserAuthSession::new(
                            app_dao.web_user.user_dao.auth_dao.clone(),
                            UserAuthToken::default(),
                        ),
                    ),
                })
            }
            None => err(JsonResponse::data(JsonData::error())
                .set_message("not find webdao")
                .into()),
        }
    }
}

//oauth 登陆实现，跟普通登陆实现方式不相同
pub struct OauthAuthQuery {
    inner: RestAuthQueryDao,
}

impl Deref for OauthAuthQuery {
    type Target = RestAuthQueryDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for OauthAuthQuery {
    type Error = ResponseJson;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user_dao_opt = req.app_data::<Data<WebDao>>();
        match user_dao_opt {
            Some(app_dao) => {
                ok(Self {
                    inner: RequestAuthDao::new(
                        RestAuthSession::new(
                            app_dao.web_app.app_dao.clone(),
                            RestAuthToken::default(),
                        ),
                    ),
                })
            }
            None => err(JsonResponse::data(JsonData::error())
                .set_message("not find webdao")
                .into()),
        }
    }
}
