mod common_test_artctps;

use common_test_artctps::{
    common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput, InitDafCfgTestinput,
};
use dev_support::artctps::common::db_pool;
use dev_support::artctps::FooOut;
use foa::{context::Cfg, db::sqlx::Db};
use sqlx::{PgPool, Postgres};

#[derive(Debug)]
struct Ctx<const K: u8> {}

impl<const K: u8> Db for Ctx<K> {
    type Database = Postgres;

    async fn pool() -> Result<PgPool, sqlx::Error> {
        db_pool().await
    }
}

mod t1 {
    use super::*;

    impl Cfg for Ctx<1> {
        type CfgInfo = CfgTestInput;

        fn cfg() -> Self::CfgInfo {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    n: "Paulo".to_owned(),
                    c: 42,
                },
                bar: BarBfCfgTestInput { incr: 11 },
                init: InitDafCfgTestinput { init_age: 10 },
            }
        }
    }

    #[tokio::test]
    async fn test1() {
        let res = common_test::<Ctx<1>>().await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<1>::cfg().foo.n,
            new_age: 1 + Ctx::<1>::cfg().init.init_age + Ctx::<1>::cfg().bar.incr,
            refresh_count: Ctx::<1>::cfg().foo.c,
        };
        assert_eq!(res_opt, Some(expected), "res={res_str}");
    }
}

mod t2 {
    use super::*;

    impl Cfg for Ctx<2> {
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

    #[ignore = "Can't run concurrently with test1"]
    #[tokio::test]
    async fn test2() {
        let res = common_test::<Ctx<2>>().await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<2>::cfg().foo.n,
            new_age: 1 + Ctx::<2>::cfg().init.init_age + Ctx::<2>::cfg().bar.incr,
            refresh_count: Ctx::<2>::cfg().foo.c,
        };
        assert_eq!(res_opt, Some(expected), "res={res_str}");
    }
}
