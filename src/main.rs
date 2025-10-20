use stefn::orquestrator::ServicesOrquestrator;
use tits::{create_api_service, create_web_service};

fn main() {
    ServicesOrquestrator::default()
        .set_config_from_env()
        .enable_migrations()
        .add_service(create_web_service())
        .add_service(create_api_service())
        .init_tracing()
        .run();
}
