use lazy_static::lazy_static;

use tera::Context;
use tera::Tera;
// use serde::Serialize;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*.*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };

    pub static ref TEMPLATES_UNESCAPED: Tera = {
        let mut tera = match Tera::new("templates/**/*.*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![]);
        tera
    };
}

// pub fn render<T: Serialize>(path: &str, s: T) -> String {
// 	TEMPLATES.render(path, &Context::from_serialize(s).expect("Unexpected serialization problem")).expect("Unexpect template error")
// }

pub fn load(path: &str) -> String {
    TEMPLATES.render(path, &Context::new()).expect("Unexpect template error")
}

pub fn render_context(path: &str, ctx: &Context) -> String {
	TEMPLATES.render(path, ctx).expect("Unexpected template error")
}

pub fn render_context_no_escapes(path: &str, ctx: &Context) -> String {
    let r = TEMPLATES_UNESCAPED.render(path, ctx).expect("Unexpected template error");
    r
}