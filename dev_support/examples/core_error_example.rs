use foa::{
    context::{ErrCtx, Locale, LocaleCtx, LocalizedMsg},
    error::CoreError,
};
use once_cell::sync::Lazy;
use std::{
    borrow::Borrow,
    collections::HashMap,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

static ERROR_DISPLAY_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("err_kind_0-en-CA", "no args"),
        ("err_kind_1-en-CA", "one arg is {} and that's it"),
        ("err_kind_2-en-CA", "two args are {} and {} and that's it"),
        ("err_kind_0-pt-BR", "nenhum parâmetro"),
        ("err_kind_1-pt-BR", "um parâmetro {} e é só"),
        ("err_kind_2-pt-BR", "dois parâmetros {} e {} e nada mais"),
    ])
});

static ERROR_KIND_0: &str = "err_kind_0";
static ERROR_KIND_1: &str = "err_kind_1";
static ERROR_KIND_2: &str = "err_kind_2";

static LOCALE_SELECTOR: AtomicUsize = AtomicUsize::new(0);
static LOCALES: [&str; 3] = ["en-CA", "pt-BR", "es-ES"];

#[derive(Debug, Clone)]
struct Ctx1;
struct SubCtx1;

impl LocalizedMsg for SubCtx1 {
    fn localized_msg<'a>(kind: &'a str, locale: impl Deref<Target = str>) -> Option<&'a str> {
        let key = kind.to_owned() + "-" + &locale;
        let raw_msg = ERROR_DISPLAY_MAP.get(&key.borrow())?;
        Some(*raw_msg)
    }
}

impl Locale for SubCtx1 {
    fn locale() -> impl Deref<Target = str> {
        LOCALES[LOCALE_SELECTOR.load(Ordering::Relaxed)]
    }
}

impl LocaleCtx for Ctx1 {
    type Locale = SubCtx1;
}

impl ErrCtx for Ctx1 {
    type LocalizedMsg = SubCtx1;
}

#[derive(Debug, Clone)]
struct Ctx2;
struct SubCtx2;

impl LocalizedMsg for SubCtx2 {
    fn localized_msg<'a>(kind: &'a str, locale: impl Deref<Target = str>) -> Option<&'a str> {
        let res = match locale.as_ref() {
            "en-CA" => match kind {
                "err_kind_0" => "no args",
                "err_kind_1" => "one arg is {} and that's it",
                "err_kind_2" => "two args are {} and {} and that's it",
                _ => return None,
            },
            "pt-BR" => match kind {
                "err_kind_0" => "nenhum parâmetro",
                "err_kind_1" => "um parâmetro {} e é só",
                "err_kind_2" => "dois parâmetros {} e {} e nada mais",
                _ => return None,
            },
            _ => return None,
        };
        Some(res)
    }
}

impl Locale for SubCtx2 {
    fn locale() -> impl Deref<Target = str> {
        LOCALES[LOCALE_SELECTOR.load(Ordering::Relaxed)]
    }
}

impl LocaleCtx for Ctx2 {
    type Locale = SubCtx2;
}
impl ErrCtx for Ctx2 {
    type LocalizedMsg = SubCtx2;
}

type MyCoreError1 = CoreError<Ctx1>;
type MyCoreError2 = CoreError<Ctx2>;

fn main() {
    println!("=================== Ctx1");
    println!();
    {
        let err0 = MyCoreError1::new(ERROR_KIND_0, vec![]);
        let err1 =
            MyCoreError1::new_with_source(ERROR_KIND_1, vec!["arg1".to_owned()], err0.clone());
        let err2 = MyCoreError1::new_with_source(
            ERROR_KIND_2,
            vec!["param1".to_owned(), "param2".to_owned()],
            err1.clone(),
        );
        let err2_lo =
            MyCoreError1::new_with_source(ERROR_KIND_2, vec!["xxx".to_owned()], err1.clone());
        let err2_hi = MyCoreError1::new_with_source(
            ERROR_KIND_2,
            vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
            err1.clone(),
        );

        for (i, locale) in LOCALES.iter().enumerate() {
            LOCALE_SELECTOR.store(i, Ordering::Relaxed);
            println!("***** Locale={}", locale);
            println!();

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

    println!("=================== Ctx2");
    println!();
    {
        let err0 = MyCoreError2::new(ERROR_KIND_0, vec![]);
        let err1 =
            MyCoreError2::new_with_source(ERROR_KIND_1, vec!["arg1".to_owned()], err0.clone());
        let err2 = MyCoreError2::new_with_source(
            ERROR_KIND_2,
            vec!["param1".to_owned(), "param2".to_owned()],
            err1.clone(),
        );
        let err2_lo =
            MyCoreError2::new_with_source(ERROR_KIND_2, vec!["xxx".to_owned()], err1.clone());
        let err2_hi = MyCoreError2::new_with_source(
            ERROR_KIND_2,
            vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
            err1.clone(),
        );

        for (i, locale) in LOCALES.iter().enumerate() {
            LOCALE_SELECTOR.store(i, Ordering::Relaxed);
            println!("***** Locale={}", locale);
            println!();

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
}
