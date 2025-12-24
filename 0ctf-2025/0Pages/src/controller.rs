use crate::model::{
    AuthResponse, ErrorResponse, LoginRequest, LogoutResponse, RegisterRequest, SiteDeleteResponse,
    SiteDeployResponse, SiteListResponse,
};
use crate::service::{
    delete_site, deploy_site, export_site, generate_site_template, get_username_from_session,
    list_sites,
};
use salvo::fs::NamedFile;
use salvo::oapi::extract::{FormFile, JsonBody, PathParam};
use salvo::prelude::Json;
use salvo::prelude::*;
use salvo::session::Session;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[endpoint]
pub async fn register_controller(
    register: JsonBody<RegisterRequest>,
    depot: &mut Depot,
    res: &mut Response,
) {
    let register = register.into_inner();
    if !register.username.chars().all(char::is_alphanumeric) {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "Username must be alphanumeric".to_string(),
        }));
        return;
    }
    if register.password != register.confirm_password {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "Passwords do not match".to_string(),
        }));
        return;
    }
    let accounts = depot
        .obtain::<Arc<RwLock<HashMap<String, String>>>>()
        .unwrap();
    let mut accounts_lock = accounts.write().unwrap();
    if accounts_lock.contains_key(&register.username) {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "Username already exists".to_string(),
        }));
        return;
    }
    accounts_lock.insert(register.username.clone(), register.password.clone());
    res.render(Json(AuthResponse {
        status: "success".to_string(),
        username: register.username.clone(),
    }));
}

#[endpoint]
pub async fn login_controller(
    login: JsonBody<LoginRequest>,
    depot: &mut Depot,
    res: &mut Response,
) {
    if let Some(username) = get_username_from_session(depot).await {
        res.render(Json(AuthResponse {
            status: "success".to_string(),
            username,
        }));
        return;
    }
    let login = login.into_inner();
    let accounts = depot
        .obtain::<Arc<RwLock<HashMap<String, String>>>>()
        .unwrap();
    let mut authenticated = false;
    {
        let accounts_lock = accounts.read().unwrap();
        if let Some(stored_password) = accounts_lock.get(&login.username) {
            if *stored_password == login.password {
                authenticated = true;
            }
        }
    }
    if authenticated {
        let mut session = Session::new();
        session.insert("username", login.username.clone()).unwrap();
        depot.set_session(session);
        res.render(Json(AuthResponse {
            status: "success".to_string(),
            username: login.username.clone(),
        }));
    } else {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "Invalid username or password".to_string(),
        }));
    }
}

#[endpoint]
pub async fn logout_controller(depot: &mut Depot, res: &mut Response) {
    if let Some(session) = depot.session_mut() {
        session.remove("username");
    }
    res.render(Json(LogoutResponse {
        status: "success".to_string(),
    }));
}

#[endpoint]
pub async fn site_list_controller(depot: &mut Depot, res: &mut Response) {
    if let Some(username) = get_username_from_session(depot).await {
        match list_sites(&username).await {
            Ok(sites) => {
                res.render(Json(SiteListResponse {
                    status: "success".to_string(),
                    sites,
                }));
            }
            Err(err) => {
                res.render(Json(ErrorResponse {
                    status: "error".to_string(),
                    message: err.to_string(),
                }));
            }
        }
    } else {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "You must be logged in to view your sites.".to_string(),
        }));
    }
}

#[endpoint]
pub async fn site_deploy_controller(archive: FormFile, depot: &mut Depot, res: &mut Response) {
    if let Some(username) = get_username_from_session(depot).await {
        match deploy_site(&username, archive.path()).await {
            Ok(manifest) => {
                res.render(Json(SiteDeployResponse {
                    status: "success".to_string(),
                    data: manifest,
                }));
            }
            Err(err) => {
                res.render(Json(ErrorResponse {
                    status: "error".to_string(),
                    message: err.to_string(),
                }));
            }
        }
    } else {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "You must be logged in to deploy a site.".to_string(),
        }));
    }
}

#[endpoint]
pub async fn site_export_controller(
    site_id: PathParam<String>,
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) {
    if let Some(username) = get_username_from_session(depot).await {
        if username != "admin" {
            res.render(Json(ErrorResponse {
                status: "error".to_string(),
                message: "You are not allowed to export site archive.".to_string(),
            }));
            return;
        }
        match export_site(&username, &site_id).await {
            Ok(archive_path) => {
                NamedFile::builder(archive_path)
                    .attached_name(format!("{}.zip", site_id))
                    .send(req.headers(), res)
                    .await;
            }
            Err(err) => {
                res.render(Json(ErrorResponse {
                    status: "error".to_string(),
                    message: err.to_string(),
                }));
            }
        }
    } else {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "You must be logged in to export site archive.".to_string(),
        }));
    }
}

#[endpoint]
pub async fn site_delete_controller(
    site_id: PathParam<String>,
    depot: &mut Depot,
    res: &mut Response,
) {
    if let Some(username) = get_username_from_session(depot).await {
        match delete_site(&username, &site_id).await {
            Ok(_) => {
                res.render(Json(SiteDeleteResponse {
                    status: "success".to_string(),
                }));
            }
            Err(err) => {
                res.render(Json(ErrorResponse {
                    status: "error".to_string(),
                    message: err.to_string(),
                }));
            }
        }
    } else {
        res.render(Json(ErrorResponse {
            status: "error".to_string(),
            message: "You must be logged in to delete a site.".to_string(),
        }));
    }
}

#[endpoint]
pub async fn site_template_controller(req: &mut Request, res: &mut Response) {
    match generate_site_template().await {
        Ok(archive_path) => {
            NamedFile::builder(archive_path)
                .attached_name("template.zip")
                .send(req.headers(), res)
                .await;
        }
        Err(err) => {
            res.render(Json(ErrorResponse {
                status: "error".to_string(),
                message: err.to_string(),
            }));
        }
    }
}
