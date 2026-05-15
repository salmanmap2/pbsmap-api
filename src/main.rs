use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::MySqlPool;
use dotenv::dotenv;
use std::env;

mod db;
mod models;
mod handlers;
mod middleware_auth;
mod utils;
mod errors;

pub struct AppState {
    pub db: MySqlPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let data = web::Data::new(AppState { db: pool });

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", host, port);

    log::info!("pbsmap-api server running at http://{}", bind_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(data.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            // ── Public Auth Routes ──
            .service(
                web::scope("/api/auth")
                    .route("/signup", web::post().to(handlers::auth::signup))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/login/google", web::post().to(handlers::auth::google_login))
                    .route("/forgot-password", web::post().to(handlers::auth::forgot_password))
                    .route("/verify-otp", web::post().to(handlers::auth::verify_otp))
                    .route("/reset-password", web::post().to(handlers::auth::reset_password))
            )
            // ── Protected User Routes ──
            .service(
                web::scope("/api/user")
                    .route("/profile", web::get().to(handlers::user::get_profile))
                    .route("/profile", web::put().to(handlers::user::update_profile))
                    .route("/change-password", web::post().to(handlers::user::change_password))
                    .route("/regenerate-api-key", web::post().to(handlers::user::regenerate_api_key))
                    .route("/join-office", web::post().to(handlers::user::join_office))
            )
            // ── Public Info Routes ──
            .service(
                web::scope("/api/public")
                    .route("/pbs-list", web::get().to(handlers::public::all_pbs_list))
                    .route("/offices/{pbs_id}", web::get().to(handlers::public::offices_by_pbs))
                    .route("/office/{office_id}", web::get().to(handlers::public::office_by_id))
                    .route("/user-by-mobile/{mobile}", web::get().to(handlers::public::user_by_mobile))
            )
            // ── Office Admin Routes (JWT required, must be office admin) ──
            .service(
                web::scope("/api/office")
                    .route("/user-change", web::post().to(handlers::office::user_change))
            )
            // ── Meter Routes ──
            .service(
                web::scope("/api/meter")
                    .route("/add", web::post().to(handlers::meter::add_meter))
                    .route("/edit", web::post().to(handlers::meter::edit_meter))
                    .route("/all", web::post().to(handlers::meter::all_meter_list))
            )
            // ── Note Routes ──
            .service(
                web::scope("/api/note")
                    .route("/add", web::post().to(handlers::note::add_note))
                    .route("/delete", web::post().to(handlers::note::delete_note))
                    .route("/all", web::post().to(handlers::note::get_all_notes))
            )
            // ── Reading Routes ──
            .service(
                web::scope("/api/reading")
                    .route("/new", web::post().to(handlers::reading::new_reading))
                    .route("/edit", web::post().to(handlers::reading::edit_reading))
                    .route("/all", web::post().to(handlers::reading::get_all_readings))
            )
            // ── Developer / Super Admin Routes ──
            .service(
                web::scope("/api/dev")
                    .route("/create-office", web::post().to(handlers::developer::create_office))
                    .route("/all-office/{pbs_id}", web::get().to(handlers::developer::all_office))
                    .route("/edit-office", web::post().to(handlers::developer::edit_office))
                    .route("/user-manage", web::post().to(handlers::developer::user_manage))
            )
    })
    .bind(&bind_addr)?
    .run()
    .await?;

    Ok(())
}
