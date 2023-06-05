use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<SubscribeFormData>, db_pool: web::Data<PgPool>) -> impl Responder {

    match sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
    Uuid::new_v4(),
    form.email,
    form.name,
    Utc::now()
    )
    // We use `get_ref` to get an immutable reference to the `PgConnection`
    // wrapped by `web::Data`.
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("Failed to execute query {}.",e);
            HttpResponse::InternalServerError()
        }
    }
}