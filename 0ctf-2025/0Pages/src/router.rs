use crate::controller;
use salvo::prelude::*;

pub fn preview_router() -> Router {
    Router::with_path("preview")
        .push(Router::with_path("{*path}").get(StaticDir::new("data").defaults("index.html")))
}

pub fn api_router() -> Router {
    let router = Router::with_path("api")
        .push(
            Router::with_path("auth")
                .push(Router::with_path("register").post(controller::register_controller))
                .push(Router::with_path("login").post(controller::login_controller))
                .push(Router::with_path("logout").post(controller::logout_controller)),
        )
        .push(
            Router::with_path("sites")
                .hoop(max_size(10 * 1024))
                .get(controller::site_list_controller)
                .post(controller::site_deploy_controller)
                .push(Router::with_path("template").get(controller::site_template_controller))
                .push(
                    Router::with_path("{site_id}")
                        .get(controller::site_export_controller)
                        .delete(controller::site_delete_controller),
                ),
        );
    let doc = OpenApi::new("0Pages API", "0.1.0").merge_router(&router);
    router
        .unshift(doc.into_router("/swagger.json"))
        .unshift(SwaggerUi::new("/api/swagger.json").into_router("/swagger-ui"))
}
