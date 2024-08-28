mod common_test_artctps;

use common_test_artctps::{common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput};
use dev_support::artctps::common::db_pool;
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

mod t1 {
    use super::*;

    #[derive(Debug)]
    struct Ctx;

    impl Cfg for Ctx {
        type CfgInfo = CfgTestInput;

        fn cfg() -> Self::CfgInfo {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    a: "Paulo".to_owned(),
                },
                bar: BarBfCfgTestInput { u: 11 },
            }
        }
    }

    impl DbCtx for Ctx {
        type Db = CtxDb;
    }

    #[tokio::test]
    async fn test1() {
        let res = common_test::<Ctx>().await;

        let expected =
            r#"Ok(FooOut { res: "foo: a=foo_test1-foo, b=4, bar=(bar: u=12, v=bar_test1-bar)" })"#;
        assert_eq!(res, Some(expected.to_owned()));
    }
}

mod t2 {
    use super::*;

    #[derive(Debug)]
    struct Ctx;

    impl Cfg for Ctx {
        type CfgInfo = CfgTestInput;

        fn cfg() -> Self::CfgInfo {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    a: "Paulo".to_owned(),
                },
                bar: BarBfCfgTestInput { u: 22 },
            }
        }
    }

    impl DbCtx for Ctx {
        type Db = CtxDb;
    }

    #[tokio::test]
    async fn test2() {
        let res = common_test::<Ctx>().await;

        let expected =
            r#"Ok(FooOut { res: "foo: a=foo_test2-foo, b=5, bar=(bar: u=23, v=bar_test2-bar)" })"#;
        assert_eq!(res, Some(expected.to_owned()));
    }
}
