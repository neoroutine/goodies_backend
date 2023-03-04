use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, http::header::ContentType};
use sqlx::sqlite::{SqlitePoolOptions, Sqlite};  
use sqlx::Pool;
use std::sync::{Arc, Mutex};

async fn get_recipes(pool: &Pool<Sqlite>) -> Vec<Recipe>
{
    match sqlx::query_as::<_, Recipe>("SELECT id, name FROM Recipe").fetch_all(pool).await
    {
        Ok(recipes) => recipes,
        Err(_) => Vec::new(),

    }
}

async fn get_recipe(pool: &Pool<Sqlite>, id: u32) -> Recipe
{
    match sqlx::query_as::<_, Recipe>("SELECT id, name FROM Recipe WHERE id = ?1").bind(id).fetch_one(pool).await
    {
        Ok(recipe) => recipe,
        Err(_) => Recipe{id: 0, name: "Error".to_string()}

    }
}

#[get("/")]
async fn app_home(data: web::Data<AppState>) -> impl Responder 
{
    format!("Welcome to {} v{}", &data.name, &data.version)
}

#[get("/recipe/{id}")]
async fn recipe_by_id(path: web::Path<u32>, data: web::Data<AppState>) -> impl Responder
{
    let id: u32 = path.into_inner();

    let recipe = get_recipe(&data.db_pool, id).await;

    format!("You requested goodie with id: {}\nName: {}", id, recipe.name)
}

#[get("/recipes")]
async fn all_recipes(data: web::Data<AppState>) -> HttpResponse
{
    let recipes = get_recipes(&data.db_pool).await;

    let mut listing = String::new();

    for recipe in recipes
    {
        listing.push_str(format!("{} ({})\n", recipe.name, recipe.id).as_str())
    }

    HttpResponse::Ok()
    .content_type(ContentType::plaintext())
    .insert_header(("Access-Control-Allow-Origin", "http://127.0.0.1:8080"))
    .body(listing)
    //format!("List of recipes:\n{}", listing)
}

struct AppState
{
    name: String,
    version: u8, 
    db_pool: Pool<Sqlite>,
}

#[derive(sqlx::FromRow)]
struct Recipe
{
    id: u32,
    name: String,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()>
{
    let test = Arc::new(Mutex::new(String::from("Test")));
    let test2 = Arc::clone(&test);

    *test2.lock().unwrap() = String::from("Test2");

    println!("Test: {:?}, test2: {:?}", test, test2);

    let pool = SqlitePoolOptions::new().max_connections(1).connect("goodies.db").await?;

    let recs = get_recipes(&pool).await;

    println!("id = {}, name = {}", recs[0].id, recs[0].name);

    let web_data = web::Data::new(AppState { name: String::from("Goodies"), version: 1, db_pool: pool});
    
    HttpServer::new(move || {
        App::new()
        .app_data(web_data.clone())
        .service(app_home)
        .service(recipe_by_id)
        .service(all_recipes)
    })
    .bind(("127.0.0.1", 8090))?
    .run()
    .await?;

    Ok(())
}