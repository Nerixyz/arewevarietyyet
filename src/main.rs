mod data_actor;
mod sullygnome;
mod model;

#[actix_web::main]
async fn main() {
    // fancy regex /^(?<game>[^|]+)\|([^|]+)\|(?<avatarUrl>.+)$/
    println!("Hello, world!");
}
