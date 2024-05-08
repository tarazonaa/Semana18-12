use std::sync::{Arc, Mutex};

use actix_web::{
    get, post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use shuttle_actix_web::ShuttleActixWeb;
use tera::Tera;

struct AppState {
    tera: Tera,
    products: Arc<Mutex<Vec<Product>>>,
}

struct Sale {
    id: u8,
    price: f32,
}

#[derive(Debug, Clone, Serialize)]
struct Product {
    id: u8,
    name: String,
    price: f32,
    stock: u8,
}

#[derive(Deserialize)]
struct FormData {
    name: String,
    price: f32,
}

impl Product {
    fn new(name: String, price: f32) -> Self {
        Product {
            id: rand::random::<u8>(),
            name,
            price,
            stock: 0,
        }
    }
}

#[get("/")]
async fn index() -> impl Responder {
    web::Redirect::to("/inventario").permanent()
}

#[get("/inventario")]
async fn inventario(data: web::Data<AppState>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Inventario");
    let guard = data.products.lock().unwrap();
    let products = guard.iter().collect::<Vec<_>>();
    ctx.insert("products", &products);
    let rendered = match data.tera.render("index.html", &ctx) {
        Ok(html) => html,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Template error: {:?}", e))
        }
    };
    HttpResponse::Ok().body(rendered)
}

#[post("/test_stream")]
async fn test_stream(data: web::Data<AppState>) -> impl Responder {
    // Generate a random number
    let num = rand::random::<u8>();
    // Return the number as a string
    let response = format!("<h1>New Sale ID: {}</h1>", num);

    HttpResponse::Ok().body(response)
}

#[get("/new_product")]
async fn new_product(data: web::Data<AppState>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("title", "Add Product");
    let rendered = match data.tera.render("product.html", &ctx) {
        Ok(html) => html,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Template error: {:?}", e))
        }
    };
    HttpResponse::Ok().body(rendered)
}

#[post("/test_add_product")]
async fn test_add_product(form: web::Form<FormData>, data: web::Data<AppState>) -> impl Responder {
    // Create a new Product
    let name = &form.name;
    let price = &form.price;
    let product = Product::new(name.to_string(), *price);
    let mut product_db = data.products.lock().unwrap_or_else(|e| e.into_inner());
    product_db.push(product);
    HttpResponse::Ok()
        .insert_header(("HX-Refresh", "true"))
        .body(format!("Product added"))
}

#[get("/products")]
async fn get_products(data: web::Data<AppState>) -> impl Responder {
    let guard = data.products.lock().unwrap();
    let products = guard.iter().collect::<Vec<_>>();
    // Return the products in the html format we want
    let response = format!(
        "<ul>{}</ul>",
        products.iter().fold(String::new(), |acc, product| {
            format!("{}<li >
    <div class=\"flex flex-col mx-auto w-96\">
      <div class=\"flex flex-row space-x-4\">
      <div class=\"bg-blue-400 h-10 w-10 items-center rounded-full text-center text-white\" id=\"product-{{ product.id }}\"><p>{}</p></div>
        <div class=\"flex flex-col space-y-2\">
          <div class=\"flex flex-row space-x-2\">
            <div class=\"font-bold\">Name:</div>
            <div>{}</div>
          </div>
          <div class=\"flex flex-row space-x-2\">
            <div class=\"font-bold\">Price:</div>
            <div>{}</div>
          </div>
          <div class=\"flex flex-row space-x-2\">
            <div class=\"font-bold\">Stock:</div>
            <div>{}</div>
          </div>
        </div>
      </div>
    </div>", acc,product.id, product.name, product.price, product.stock)
        },)
    );
    HttpResponse::Ok().body(response)
}

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let tera = match Tera::new("./static/**/*.html") {
        Ok(t) => t,
        Err(e) => panic!("Error parsing templates: {:?}", e),
    };
    let product_db = Arc::new(Mutex::new(vec![
        Product::new("Burger".to_string(), 1.0),
        Product::new("Sprite".to_string(), 0.5),
        Product::new("Fries".to_string(), 1.5),
    ]));

    let app_state = web::Data::new(AppState {
        products: product_db,
        tera: tera.clone(),
    });

    let config = move |cfg: &mut web::ServiceConfig| {
        cfg.app_data(app_state.clone())
            .service(index)
            .service(inventario)
            .service(test_stream)
            .service(new_product)
            .service(test_add_product)
            .service(get_products);
    };

    Ok(config.into())
}
