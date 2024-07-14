use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use foa::{CoreError, ErrorDisplayMap, Locale};
use once_cell::sync::Lazy;

static ERROR_DISPLAY_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("err_kind_0-en-ca", "no args"),
        ("err_kind_1-en-ca", "one arg is {} and that's it"),
        ("err_kind_2-en-ca", "two args are {} and {} and that's it"),
        ("err_kind_0-pt-br", "nenhum parâmetro"),
        ("err_kind_1-pt-br", "um parâmetro {} e é só"),
        ("err_kind_2-pt-br", "dois parâmetros {} e {} e nada mais"),
    ])
});

const ERROR_KIND_0: &str = "err_kind_0";
const ERROR_KIND_1: &str = "err_kind_1";
const ERROR_KIND_2: &str = "err_kind_2";

#[derive(Debug, Clone)]
struct Ctx;

impl ErrorDisplayMap for Ctx {
    fn display_map<'a>() -> &'a HashMap<&'a str, &'a str> {
        &ERROR_DISPLAY_MAP
    }
}

static LOCALE_SELECTOR: AtomicUsize = AtomicUsize::new(0);
const LOCALES: [&str; 3] = ["en-ca", "pt-br", "es-es"];

impl Locale for Ctx {
    fn locale<'a>() -> &'a str {
        LOCALES[LOCALE_SELECTOR.load(Ordering::Relaxed)]
    }
}

type MyCoreError = CoreError<Ctx>;

fn main() {
    for i in 0..3 {
        LOCALE_SELECTOR.store(i, Ordering::Relaxed);
        println!("***** Locale={}", LOCALES[i]);
        println!();

        let err0 = MyCoreError::new(&ERROR_KIND_0, vec![]);
        let err1 =
            MyCoreError::new_with_source(&ERROR_KIND_1, vec!["arg1".to_owned()], err0.clone());
        let err2 = MyCoreError::new_with_source(
            &ERROR_KIND_2,
            vec!["param1".to_owned(), "param2".to_owned()],
            err1.clone(),
        );
        let err2_lo =
            MyCoreError::new_with_source(&ERROR_KIND_2, vec!["xxx".to_owned()], err1.clone());
        let err2_hi = MyCoreError::new_with_source(
            &ERROR_KIND_2,
            vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
            err1.clone(),
        );

        println!("err0={err0:?}");
        println!("err0 msg={err0}");
        println!();

        println!("err1={err1:?}");
        println!("err1 msg={err1}");
        println!();

        println!("err2={err2:?}");
        println!("err2 msg={err2}");
        println!();

        println!("err2_lo={err2_lo:?}");
        println!("err2_lo msg={err2_lo}");
        println!();

        println!("err2_hi={err2_hi:?}");
        println!("err2_hi msg={err2_hi}");
        println!();
    }
}
