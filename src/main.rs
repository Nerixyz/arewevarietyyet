use crate::{
    data_actor::{DataActor, GetData},
    model::Year,
};
use actix::{Actor, Recipient};
use actix_files::Files;
use actix_web::{error, get, http::header::ContentType, web, App, HttpResponse, HttpServer};
use handlebars::{DirectorySourceOptions, Handlebars};
use model::StreamerModel;
use serde::Serialize;
use std::io;

mod data_actor;
mod datetime;
mod helpers;
mod model;
mod streamcounter;
mod sullygnome;

#[derive(Serialize)]
struct TemplateContext<'a> {
    streamer: &'a StreamerModel,
    years: &'a Vec<i32>,
    child: &'static str,
}

async fn render_template(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
    year: Year,
) -> Result<HttpResponse, actix_web::Error> {
    let (streamer, years) = actor
        .send(GetData(year))
        .await
        .map_err(error::ErrorTooManyRequests)?
        .map_err(error::ErrorInternalServerError)?;
    let ctx = TemplateContext {
        streamer: &streamer,
        years: &years,
        child: match year {
            Year::Current => "this-year",
            Year::Last(_) => "last-year",
        },
    };
    let rendered = handlebars
        .render("skeleton", &ctx)
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

#[get("/prev/{year}")]
async fn last_year(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
    path: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    render_template(actor, handlebars, Year::Last(path.into_inner())).await
}

#[get("/custom-api")]
async fn custom_api(
    actor: web::Data<Recipient<GetData>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (model, _) = actor
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
        .register_templates_directory("templates", {
            let mut opts = DirectorySourceOptions::default();
            opts.tpl_extension = ".hbs.html".to_owned();
            opts
        })
        .unwrap();
    handlebars.register_helper("bar-width", Box::new(helpers::bar_width));
    handlebars.register_helper("humanize-min", Box::new(helpers::humanize_min));
    handlebars.register_helper("round-percent", Box::new(helpers::rounded_percent));
    handlebars.register_helper("format-hours", Box::new(helpers::format_hours));
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
