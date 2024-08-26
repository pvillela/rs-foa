mod common_test_artctps;

use common_test_artctps::{common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput};
use dev_support::artctps::common::pool_tx;
use foa::context::{Cfg, CfgCtx, Itself};
use foa::db::sqlx::pg::Db;
use sqlx::{Postgres, Transaction};

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
            pool_tx(self).await
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

        let expected =
            r#"Ok(FooOut { res: "foo: a=foo_test1-foo, b=4, bar=(bar: u=12, v=bar_test1-bar)" })"#;
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
            pool_tx(self).await
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

        let expected =
            r#"Ok(FooOut { res: "foo: a=foo_test2-foo, b=5, bar=(bar: u=23, v=bar_test2-bar)" })"#;
        assert_eq!(res, Some(expected.to_owned()));
    }
}
