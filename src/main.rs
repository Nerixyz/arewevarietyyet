use crate::data_actor::{DataActor, GetData};
use actix::{Actor, Recipient};
use actix_files::Files;
use actix_web::{error, http::header::ContentType, web, App, HttpResponse, HttpServer};
use handlebars::Handlebars;
use std::io;

mod data_actor;
mod helpers;
mod model;
mod sullygnome;

async fn index(
    actor: web::Data<Recipient<GetData>>,
    handlebars: web::Data<Handlebars<'_>>,
) -> Result<HttpResponse, actix_web::Error> {
    let model = actor
        .send(GetData)
        .await
        .map_err(|e| error::ErrorTooManyRequests(e))?
        .map_err(|e| error::ErrorInternalServerError(e))?;
    let rendered = handlebars
        .render("index", &*model)
        .map_err(|e| error::ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok()
        .insert_header(ContentType::html())
        .body(rendered))
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
    let handlebars = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(actor.clone())
            .app_data(handlebars.clone())
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
