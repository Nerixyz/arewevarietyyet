use crate::{
    data_actor::{DataActor, GetData},
    model::Year,
};
use actix::{Actor, Recipient};
use actix_files::Files;
use actix_web::{error, get, http::header::ContentType, web, App, HttpResponse, HttpServer};
use handlebars::Handlebars;
use std::io;

mod data_actor;
mod helpers;
mod model;
mod streamcounter;
mod sullygnome;

async fn render_template(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
    year: Year,
) -> Result<HttpResponse, actix_web::Error> {
    let model = actor
        .send(GetData(year))
        .await
        .map_err(error::ErrorTooManyRequests)?
        .map_err(error::ErrorInternalServerError)?;
    let rendered = handlebars
        .render(
            match year {
                Year::Current => "index",
                Year::Last => "last-year",
            },
            &*model,
        )
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(rendered))
}

async fn index(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, actix_web::Error> {
    render_template(actor, handlebars, Year::Current).await
}

#[get("/last-year")]
async fn last_year(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, actix_web::Error> {
    render_template(actor, handlebars, Year::Last).await
}

#[get("/custom-api")]
async fn custom_api(
    actor: web::Data<Recipient<GetData>>,
) -> Result<HttpResponse, actix_web::Error> {
    let model = actor
        .send(GetData(Year::Current))
        .await
        .map_err(|_| {
            actix_web::error::ErrorTooManyRequests("🤯 Actor mailbox closed or we timed out.")
        })?
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("🚨 API failed: {e}")))?;
    Ok(HttpResponse::Ok()
        .insert_header(ContentType::plaintext())
        .body(format!(
            "{prefix} {p_variety}% variety this year. {n_days_ditched}/{n_days} days ({p_days_ditched}%) ditched.",
            prefix = match model.are_we_variety {
                true => "Yes,",
                false => "No, we only had",
            },
            p_variety = (model.variety_percent * 100.0).round(),
            n_days_ditched = model.days_ditched,
            n_days = model.days_until_now,
            p_days_ditched = (model.percent_ditched * 100.0).round()
        )))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let actor = DataActor::start_default();
    let actor = web::Data::new(actor.recipient::<GetData>());
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".hbs.html", "templates")
        .unwrap();
    handlebars.register_helper("bar-width", Box::new(helpers::bar_width));
    handlebars.register_helper("humanize-min", Box::new(helpers::humanize_min));
    handlebars.register_helper("round-percent", Box::new(helpers::rounded_percent));
    let handlebars = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(actor.clone())
            .app_data(handlebars.clone())
            .service(web::scope("/api").service(custom_api))
            .service(last_year)
            .service(
                Files::new("/", "static")
                    .index_file("this_file_doesnt_exist_but_we_dont_need_it")
                    .default_handler(web::route().to(index)),
            )
    })
    .bind("127.0.0.1:8934")?
    .run()
    .await
}
