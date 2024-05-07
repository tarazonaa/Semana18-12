use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use shuttle_actix_web::ShuttleActixWeb;
use tera::Tera;

struct AppState {
    tera: Tera,
}

#[get("/")]
async fn index() -> impl Responder {
    web::Redirect::to("/inventario").permanent()
}

#[get("/inventario")]
async fn inventario(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Inventario");
    let rendered = match data.tera.render("index.html", &ctx) {
        Ok(html) => html,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Template error: {:?}", e))
        }
    };
    HttpResponse::Ok().body(rendered)
}

#[post("/testpost")]
async fn testpost() -> impl Responder {
    let response = "<h1>Test Post</h1>";
    HttpResponse::Ok().body(response)
}

#[get("/clicked")]
async fn clicked() -> impl Responder {
    let response = "<h1>Clicked</h1>";
    HttpResponse::Ok().body(response)
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let tera = match Tera::new("./static/**/*.html") {
        Ok(t) => t,
        Err(e) => panic!("Error parsing templates: {:?}", e),
    };
    let app_state = web::Data::new(AppState { tera: tera.clone() });

    let config = move |cfg: &mut web::ServiceConfig| {
        cfg.app_data(app_state.clone())
            .service(index)
            .service(inventario)
            .service(clicked)
            .service(testpost);
    };

    Ok(config.into())
}
