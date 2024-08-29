mod common_test_artctps;

use common_test_artctps::{
    common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput, InitDafCfgTestinput,
};
use dev_support::artctps::common::db_pool;
use dev_support::artctps::FooOut;
use foa::{
    context::{Cfg, DbCtx},
    db::sqlx::pg::Db,
};
use sqlx::PgPool;

struct CtxDb;

impl Db for CtxDb {
    async fn pool() -> Result<PgPool, sqlx::Error> {
        db_pool().await
    }
}

// mod t1 {

//     use super::*;

//     #[derive(Debug, PartialEq)]
//     struct Ctx;

//     impl Cfg for Ctx {
//         type CfgInfo = CfgTestInput;

//         fn cfg() -> Self::CfgInfo {
//             CfgTestInput {
//                 foo: FooSflCfgTestInput {
//                     n: "Paulo".to_owned(),
//                     c: 42,
//                 },
//                 bar: BarBfCfgTestInput { incr: 11 },
//                 init: InitDafCfgTestinput { init_age: 10 },
//             }
//         }
//     }

//     impl DbCtx for Ctx {
//         type Db = CtxDb;
//     }

//     #[tokio::test]
//     async fn test1() {
//         let res = common_test::<Ctx>().await;
//         let res_str = format!("{res:?}");
//         let res_opt = res.ok();

//         let expected = FooOut {
//             name: Ctx::cfg().foo.n,
//             new_age: 1 + Ctx::cfg().init.init_age + Ctx::cfg().bar.incr,
//             refresh_count: Ctx::cfg().foo.c,
//         };
//         assert_eq!(res_opt, Some(expected), "res={res_str}");
//     }
// }

mod t2 {
    use super::*;

    #[derive(Debug)]
    struct Ctx;

    impl Cfg for Ctx {
        type CfgInfo = CfgTestInput;

        fn cfg() -> Self::CfgInfo {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    n: "Paulo".to_owned(),
                    c: 99,
                },
                bar: BarBfCfgTestInput { incr: 100 },
                init: InitDafCfgTestinput { init_age: 1 },
            }
        }
    }

    impl DbCtx for Ctx {
        type Db = CtxDb;
    }

    #[tokio::test]
    async fn test2() {
        let res = common_test::<Ctx>().await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::cfg().foo.n,
            new_age: 1 + Ctx::cfg().init.init_age + Ctx::cfg().bar.incr,
            refresh_count: Ctx::cfg().foo.c,
        };
        assert_eq!(res_opt, Some(expected), "res={res_str}");
    }
}
