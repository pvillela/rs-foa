mod common_test_artctps;

use common_test_artctps::{BarBfCfgTestInput, CfgTestInput, common_test, FooSflCfgTestInput};
use foa::context::{Cfg, CfgCtx, Itself};
use foa::db::sqlx::pg::Db;
use sqlx::{PgPool, Postgres, Transaction};
use tokio;

mod t1 {
    use super::*;

    #[derive(Debug)]
    struct Ctx;
    struct CtxCfg;

    impl Cfg for CtxCfg {
        type Info = CfgTestInput;

        fn cfg() -> Self::Info {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    a: "foo_test1".to_owned(),
                    b: 1,
                },
                bar: BarBfCfgTestInput {
                    u: 11,
                    v: "bar_test1".to_owned(),
                },
            }
        }
    }

    impl CfgCtx for Ctx {
        type Cfg = CtxCfg;
    }

    impl Db for Ctx {
        async fn pool_tx<'c>(&'c self) -> Result<Transaction<'c, Postgres>, sqlx::Error> {
            let pool =
                PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
            pool.begin().await.map_err(|err| err.into())
        }
    }

    impl Itself<Ctx> for Ctx {
        fn itself() -> Ctx {
            Ctx
        }
    }

    #[tokio::test]
    async fn test1() {
        let res = common_test::<Ctx>().await;

        let expected = r#"Ok(FooOut { res: "foo: a=foo_test1-foo, b=4, bar=(bar: u=12, v=bar_test1-bar-Tx.dummy() called from bar_bf_c)-Tx.dummy() called from foo_sfl_c" })"#;
        assert_eq!(res, Some(expected.to_owned()));
    }
}

mod t2 {
    use super::*;

    #[derive(Debug)]
    struct Ctx;
    struct CtxCfg;

    impl Cfg for CtxCfg {
        type Info = CfgTestInput;

        fn cfg() -> Self::Info {
            CfgTestInput {
                foo: FooSflCfgTestInput {
                    a: "foo_test2".to_owned(),
                    b: 2,
                },
                bar: BarBfCfgTestInput {
                    u: 22,
                    v: "bar_test2".to_owned(),
                },
            }
        }
    }

    impl CfgCtx for Ctx {
        type Cfg = CtxCfg;
    }

    impl Db for Ctx {
        async fn pool_tx<'c>(&'c self) -> Result<Transaction<'c, Postgres>, sqlx::Error> {
            let pool =
                PgPool::connect("postgres://testuser:testpassword@localhost:9999/testdb").await?;
            pool.begin().await.map_err(|err| err.into())
        }
    }

    impl Itself<Ctx> for Ctx {
        fn itself() -> Ctx {
            Ctx
        }
    }

    #[tokio::test]
    async fn test2() {
        let res = common_test::<Ctx>().await;

        let expected = r#"Ok(FooOut { res: "foo: a=foo_test2-foo, b=5, bar=(bar: u=23, v=bar_test2-bar-Tx.dummy() called from bar_bf_c)-Tx.dummy() called from foo_sfl_c" })"#;
        assert_eq!(res, Some(expected.to_owned()));
    }
}
