mod common_test_artctpg;

use axum::http::{request, request::Parts, Method, Uri, Version};
use common_test_artctpg::{
    common_test, BarBfCfgTestInput, CfgTestInput, FooSflCfgTestInput, InitDafCfgTestinput,
};
use dev_support::artctpg::{common::new_db_pool, FooOut};
use foa::{
    context::{Cfg, Locale, LocaleCtx},
    db::sqlx::{Db, DbCtx},
    tokio::{
        task_local::{TaskLocal, TaskLocalCtx},
        task_local_ext::locale_from_task_local,
    },
};
use sqlx::{PgPool, Postgres};
use tokio::task::LocalKey;

#[derive(Debug)]
struct Ctx<const K: u8>;
struct SubCtx<const K: u8>;

impl<const K: u8> Db for SubCtx<K> {
    type Database = Postgres;

    async fn pool() -> Result<PgPool, sqlx::Error> {
        new_db_pool().await
    }
}

impl<const K: u8> DbCtx for Ctx<K> {
    type Db = SubCtx<K>;
}

impl<const K: u8> LocaleCtx for Ctx<K> {
    type Locale = SubCtx<K>;
}

tokio::task_local! {
    static CTX_TL: Parts;
}

impl<const K: u8> TaskLocal for SubCtx<K> {
    type ValueType = Parts;

    fn local_key() -> &'static LocalKey<Self::ValueType> {
        &CTX_TL
    }
}

impl<const K: u8> TaskLocalCtx for Ctx<K> {
    type TaskLocal = SubCtx<K>;
}

impl<const K: u8> Locale for SubCtx<K> {
    fn locale() -> impl std::ops::Deref<Target = str> + Send {
        locale_from_task_local::<Self>("en-CA")
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

    pub(super) async fn test_serial() {
        let (parts, _) = {
            let req = request::Builder::new()
                .header("Accept-Language", "pt-BR")
                .method(Method::PUT)
                .uri(Uri::from_static("foo.com"))
                .version(Version::HTTP_2)
                .body(())
                .unwrap();
            req.into_parts()
        };
        let res = common_test::<Ctx<1>>(parts.clone()).await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<1>::cfg().foo.n,
            new_age: 1 + Ctx::<1>::cfg().init.init_age + Ctx::<1>::cfg().bar.incr,
            refresh_count: Ctx::<1>::cfg().foo.c,
            locale: "pt-BR".into(),
            method: parts.method.to_string(),
            extensions: format!("{:?}", parts.extensions),
            uri: parts.uri.to_string(),
            version: format!("{:?}", parts.version),
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
        let (parts, _) = {
            let req = request::Builder::new()
                .header("Accept-Language", "es-ES")
                .method(Method::GET)
                .uri(Uri::from_static("bar.com"))
                .version(Version::HTTP_11)
                .body(())
                .unwrap();
            req.into_parts()
        };
        let res = common_test::<Ctx<2>>(parts.clone()).await;
        let res_str = format!("{res:?}");
        let res_opt = res.ok();

        let expected = FooOut {
            name: Ctx::<2>::cfg().foo.n,
            new_age: 1 + Ctx::<2>::cfg().init.init_age + Ctx::<2>::cfg().bar.incr,
            refresh_count: Ctx::<2>::cfg().foo.c,
            locale: "es-ES".into(),
            method: parts.method.to_string(),
            extensions: format!("{:?}", parts.extensions),
            uri: parts.uri.to_string(),
            version: format!("{:?}", parts.version),
        };
        assert_eq!(res_opt, Some(expected), "res={res_str}");
    }
}

#[tokio::test]
async fn test_serial_1_2() {
    t1::test_serial().await;
    t2::test_serial().await;
}
