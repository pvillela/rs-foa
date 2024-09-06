mod common_test_artctpg;

use common_test_artctpg::{
    common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput, InitDafCfgTestinput,
};
use dev_support::artctpg::common::new_db_pool;
use dev_support::artctpg::FooOut;
use foa::{
    context::{Cfg, Locale, LocaleCtx},
    db::sqlx::{Db, DbCtx},
};
use sqlx::{PgPool, Postgres};

#[derive(Debug)]
struct Ctx<const K: u8, const L: u8 = 0> {}

impl<const K: u8> Db for Ctx<K, 1> {
    type Database = Postgres;

    async fn pool() -> Result<PgPool, sqlx::Error> {
        new_db_pool().await
    }
}

impl<const K: u8> DbCtx for Ctx<K> {
    type Db = Ctx<K, 1>;
}

impl<const K: u8> Locale for Ctx<K, 1> {
    fn locale() -> impl std::ops::Deref<Target = str> + Send {
        "en-CA"
    }
}

impl<const K: u8> LocaleCtx for Ctx<K> {
    type Locale = Ctx<K, 1>;
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

    pub(super) async fn test_serial() {
        let res = common_test::<Ctx<1>>().await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<1>::cfg().foo.n,
            new_age: 1 + Ctx::<1>::cfg().init.init_age + Ctx::<1>::cfg().bar.incr,
            refresh_count: Ctx::<1>::cfg().foo.c,
            locale: "en-CA".into(),
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

    pub(super) async fn test_serial() {
        let res = common_test::<Ctx<2>>().await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<2>::cfg().foo.n,
            new_age: 1 + Ctx::<2>::cfg().init.init_age + Ctx::<2>::cfg().bar.incr,
            refresh_count: Ctx::<2>::cfg().foo.c,
            locale: "en-CA".into(),
        };
        assert_eq!(res_opt, Some(expected), "res={res_str}");
    }
}

#[tokio::test]
async fn test_serial_1_2() {
    t1::test_serial().await;
    t2::test_serial().await;
}