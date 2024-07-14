use std::{collections::HashMap, error::Error as StdError, sync::Arc};

use foa::{CoreError, NoDebug};
use once_cell::sync::Lazy;

static ERROR_DISPLAY_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (ERROR_KIND_0, "no args"),
        (ERROR_KIND_1, "one arg is {} and that's it"),
        (ERROR_KIND_2, "two args are {} and {} and that's it"),
    ])
});

fn new_core_error(
    kind: &'static str,
    args: Vec<String>,
    source: Option<Arc<dyn StdError>>,
) -> CoreError {
    CoreError {
        kind,
        args,
        source,
        display_map: NoDebug(&ERROR_DISPLAY_MAP),
    }
}

const ERROR_KIND_0: &str = "err_kind_0";
const ERROR_KIND_1: &str = "err_kind_1";
const ERROR_KIND_2: &str = "err_kind_2";

fn main() {
    let err0 = new_core_error(&ERROR_KIND_0, vec![], None);
    let err1 = new_core_error(
        &ERROR_KIND_1,
        vec!["arg1".to_owned()],
        Some(Arc::new(err0.clone())),
    );
    let err2 = new_core_error(
        &ERROR_KIND_2,
        vec!["param1".to_owned(), "param2".to_owned()],
        Some(Arc::new(err1.clone())),
    );
    let err2_lo = new_core_error(
        &ERROR_KIND_2,
        vec!["xxx".to_owned()],
        Some(Arc::new(err1.clone())),
    );
    let err2_hi = new_core_error(
        &ERROR_KIND_2,
        vec!["x".to_owned(), "y".to_owned(), "z".to_owned()],
        Some(Arc::new(err1.clone())),
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
